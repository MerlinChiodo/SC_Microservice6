use std::path::PathBuf;
use actix_web::{Either, Responder, web};
use actix_web::cookie::Cookie;
use actix_web::error::Kind::Http;
use actix_web::http::{HeaderValue, StatusCode};
use actix_web::web::{Data, HttpResponse};
use lettre::smtp::authentication::Mechanism::Login;
use moon::actix_files::NamedFile;
use reqwest::header::LOCATION;
use crate::auth::Actions::{check_user_session_token, get_employee_info, login_employee, login_user, register_employee, register_user, verify_employee};
use crate::auth::Citizen::IsCitizen;
use crate::auth::Credentials::CredentialsPair;
use crate::auth::Employee::NewEmployeeInfo;
use crate::auth::Errors::{DatabaseError, IntoHttpError, LoginError, LoginResult, SessionRetrievalError, SessionRetrievalResult, UserRegistrationError};
use crate::auth::Request::{EmployeeInfoRequestResponse, EmployeeLoginRequestResponse, EmployeeRegisterRequest, ExternalUserLoginRequest, TokenValidateRequest, UserInfoRequestResponse, UserLoginRequest, UserLoginRequestResponse, UserRegistrationRequest};
use crate::server::DBPool;

pub async fn user_register(pool: Data<DBPool>, request: web::Form<UserRegistrationRequest>) -> Result<HttpResponse, UserRegistrationError> {
    let redirect_error = request.redirect_error.clone();
    let redirect_success = request.redirect_success.clone();
    let insert_user = {
        let db = pool.get().map_err(|_| UserRegistrationError::Db(DatabaseError::Connection))?;
        register_user(&db, &request.into_inner())
    };
    return match web::block(|| insert_user).await? {
        Err(e) => {
            redirect_error.map_or_else(|| Err(e), |url| Ok(HttpResponse::Found().append_header((LOCATION, HeaderValue::try_from(url).unwrap())).finish()))
        }
        Ok(()) => {
            redirect_success.map_or_else(|| Ok(HttpResponse::Found().append_header((LOCATION, HeaderValue::try_from("/page/login").unwrap())).finish()),
                                                 |url| Ok(HttpResponse::Found().append_header((LOCATION, HeaderValue::try_from(url).unwrap())).finish()))
        }
    };
}

pub async fn user_login(pool: Data<DBPool>, request: web::Form<UserLoginRequest>) -> Result<HttpResponse, LoginError> {
    let db = pool.get().map_err(|_| LoginError::Db(DatabaseError::Connection))?;
    let redirect_success = request.redirect_success.clone();
    let redirect_error = request.redirect_error.clone();

    let request = request.into_inner();

    let result = web::block(move || login_user(&db, &request))
        .await?;

    let result = match result {
        Err(e) => {
            return redirect_error.map_or_else(|| Err(e), |url| Ok(HttpResponse::Found().append_header((LOCATION, HeaderValue::try_from(url).unwrap())).finish()));
        }
        Ok(r) => r
    };

    let response = UserInfoRequestResponse {
        citizen_id: result.user.id.clone(),
        username: result.user.username.clone(),
        user_session_token: result.new_session_token,
        info: result.user.get_citizen_info().await?
    };

    let cookie = Cookie::build("user_session_token", response.user_session_token.clone())
        .domain("supersmartcity.de")
        .finish();

    return redirect_success.map_or_else(|| Ok(HttpResponse::Ok().cookie(cookie.clone()).json(response.clone())), |url| {
        Ok(HttpResponse::Found()
            .append_header((LOCATION, HeaderValue::try_from(format!("{}?token={}", url, &response.user_session_token)).unwrap()))
            .cookie(cookie.clone())
            .finish())
    });
}

pub async fn user_verify(pool: Data<DBPool>, request: web::Form<TokenValidateRequest>) -> Result<HttpResponse, SessionRetrievalError> {
    let check_token_from_request = {
        let db = pool.get().map_err(|_| SessionRetrievalError::Db(DatabaseError::Connection))?;
        let code = &request.code;
        check_user_session_token(&db, code)
    };

    let user = web::block(|| check_token_from_request)
        .await??;

    Ok(HttpResponse::Ok()
        .json(UserInfoRequestResponse {
            citizen_id: user.id,
            user_session_token: request.code.clone(),
            info: user.get_citizen_info().await?,
            username: user.username,
        }))
}

pub async fn login_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open(PathBuf::from(r"static_content/login_example.html")).unwrap())
}

pub async fn login_external(_: web::Query<ExternalUserLoginRequest>) -> impl Responder {
    NamedFile::open(PathBuf::from(r"static_content/login_example.html")).unwrap()

}
pub async fn employee_login_external(_: web::Query<ExternalUserLoginRequest>) -> impl Responder {
    NamedFile::open(PathBuf::from(r"static_content/login_example_employee.html")).unwrap()

}

pub async fn employee_register(pool: web::Data<DBPool>, data: web::Form<EmployeeRegisterRequest>) -> Result<HttpResponse, UserRegistrationError> {
    let data = data.into_inner();
    let db = pool.get().map_err(|e| UserRegistrationError::Db(DatabaseError::Connection))?;

    if !(&data.code == "ROOT") {
        let user_verification = move || {
            verify_employee(&db, &data.code.clone())
        };

        web::block(user_verification)
            .await??;
    };

    let user_creation = move || {
        let db = pool.clone()
            .get()
            .map_err(|e| UserRegistrationError::Db(DatabaseError::Connection))?;
        register_employee(&db, &data.info, &data.credentials)
    };
    web::block(user_creation).await??;

    Ok(HttpResponse::Ok().finish())
}

pub async fn employee_login(pool: web::Data<DBPool>, credentials: web::Form<UserLoginRequest>) -> LoginResult<HttpResponse> {
    let db = pool.get().map_err(|_| LoginError::Db(DatabaseError::Connection))?;
    let redirect_error = credentials.redirect_error.clone();
    let redirect_success = credentials.redirect_success.clone();
    let login_response = match web::block(move || login_employee(&db, &credentials.credentials)).await? {
        Err(e) => {
            return redirect_error.map_or_else(|| Err(e), |url| Ok(HttpResponse::Found().append_header((LOCATION, HeaderValue::try_from(url).unwrap())).finish()));
        },
        Ok(r) => r
    };

    let username = login_response.employee.username.clone();
    let e_id = login_response.employee.id;
    let get_info = move ||  {
        let db = pool.get().map_err(|_| LoginError::Db(DatabaseError::Connection))?;
        get_employee_info(&db, &login_response.employee).map_err(|e| LoginError::SessionRetrieval(e.into()))
    };

    let info = web::block(get_info).await??;
    let response = EmployeeInfoRequestResponse {
        id: e_id,
        username,
        employee_session_token: login_response.new_employee_token.clone(),
        info: NewEmployeeInfo {firstname: info.firstname, lastname: info.lastname}
    };

    let cookie = Cookie::build("employee_session_token", response.employee_session_token.clone())
        .domain("supersmartcity.de")
        .finish();
    redirect_success.map_or_else(|| {
        Ok(HttpResponse::Ok()
            .cookie(cookie.clone())
            .json(response.clone()))
    }, |url| {
        let http_response = HttpResponse::Found()
            .append_header((LOCATION, HeaderValue::try_from(format!("{}?token={}", url, &response.employee_session_token)).unwrap()))
            .cookie(cookie.clone())
            .finish();
        Ok(http_response)
    })
}

pub async fn employee_verify(pool: web::Data<DBPool>, token: web::Form<TokenValidateRequest>) -> SessionRetrievalResult<HttpResponse> {
    let db = pool.get().map_err(|_| SessionRetrievalError::Db(DatabaseError::Connection))?;
    let token = token.into_inner().code;

    let verify_result = web::block(move || verify_employee(&db, &token)).await??;

    let e_id = verify_result.employee.id.clone();
    let username = verify_result.employee.username.clone();

    let get_info = move ||  {
        let db = pool.get().map_err(|_| SessionRetrievalError::Db(DatabaseError::Connection))?;
        get_employee_info(&db, &verify_result.employee)
    };

    let info = web::block(get_info)
        .await??;

    let response = EmployeeInfoRequestResponse {
        id: e_id,
        username,
        employee_session_token: verify_result.new_employee_token.clone(),
        info: NewEmployeeInfo {firstname: info.firstname, lastname: info.lastname}
    };

    Ok(HttpResponse::Ok().json(response))

}