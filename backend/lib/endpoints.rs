use std::path::PathBuf;
use actix_web::{HttpResponse, Responder, web};
use actix_web::cookie::Cookie;
use actix_web::error::BlockingError;
use actix_web::error::Kind::Http;
use crate::actions::{check_pending_user_token, check_token, create_employee, get_employee_info, get_session, get_token, get_user, insert_new_session, insert_new_user, login_employee, SessionRetrieveError, UserAuthError, verify_employee};
use crate::models::{EmployeeLogin, ExternalUserLoginRequest, NewEmployeeInfo, NewEmployeeSession, UserLoginRequest};
use crate::server::DBPool;
use actix_web::post;
use moon::actix_files::NamedFile;
use crate::error::RegistrationRequestError;
use crate::request::{RegistrationRequest, Request};
use crate::schema::Users::username;
use crate::user::{CitizenInfo, UserInfo};
use actix_web::get;
use actix_web::http::HeaderValue;
use actix_web::web::to;
use reqwest::header::LOCATION;
use serde_json::Value;
use serde::Deserialize;
use serde::Serialize;
pub async fn register(pool: web::Data<DBPool>,
                      request: web::Form<RegistrationRequest>) -> Result<HttpResponse, RegistrationRequestError> {
    let db = pool.get()
        .map_err(|_|RegistrationRequestError::ServerError)?;
    let request = request.into_inner();
    let code = request.code.clone();

    let pending_user = web::block(move || check_pending_user_token(&db, &code))
        .await
        .map_err(|_| RegistrationRequestError::ServerError)?
        .map_err(|_| RegistrationRequestError::InvalidCitizenToken)?;


    let user_identity = UserInfo {
        username: request.info.username.clone(),
        password: request.info.password.clone()
    };

    let result = web::block(move || insert_new_user(&pool.get().unwrap(), user_identity, pending_user.citizen as u64))
        .await
        .map_err(|_| RegistrationRequestError::ServerError)?;

    //TODO: Maybe let the client handle the redirect in the future
    match result {
        Ok(_) => Ok(request.get_success_response()),
        Err(_) => Ok(request.get_error_response())
    }
}

pub async fn login_external(request: web::Query<ExternalUserLoginRequest>) -> impl Responder {
    println!("Hey!");
    NamedFile::open(PathBuf::from(r"static_content/login_example.html")).unwrap()

}
pub async fn employee_login_external(request: web::Query<ExternalUserLoginRequest>) -> impl Responder {
    println!("Hey!");
    NamedFile::open(PathBuf::from(r"static_content/login_example_employee.html")).unwrap()

}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserInfoRequestResponse {
    citizen_id: u64,
    username: String,
    user_session_token: String,
    info: CitizenInfo
}
pub async fn login(pool: web::Data<DBPool>, request: web::Form<UserLoginRequest>) -> Result<HttpResponse, UserAuthError> {

    let request = request.into_inner();
    let user_info = UserInfo {
        username: request.username.clone(),
        password: request.password.clone()
    };
    let db = pool.get()
        .map_err(|err| UserAuthError::ServerError)?;

    let user = web::block(move || get_user(&db, &user_info))
        .await
        .map_err(|e| UserAuthError::ServerError)?;

    if let Err(e) = user {
        return Ok(request.get_error_response())
    };

    //THIS IS BAD


    //TODO: Check if a session already exists and reuse that one (?)
    let get_token_from_request = {
        let db = pool.get().map_err(|err| UserAuthError::ServerError)?;
        get_token(&db,&user.as_ref().unwrap())
    };

    let mut token= web::block(|| get_token_from_request)
        .await
        .map_err(|e| UserAuthError::ServerError)?;


    //TODO:  Yeah at this point unwraping would actually be safer

    let insert_token_from_request = {
        let db = pool.get().unwrap();
        insert_new_session(&db, &user.as_ref().unwrap())
    };

    if let Err(e) = token {
        token = web::block(|| insert_token_from_request)
            .await
            .map_err(|e| UserAuthError::ServerError)?
            .map_err(|e| SessionRetrieveError::NoSessionFound)
    }

    if let Err(_) = token {
        if let None = &request.redirect_error {
            return Ok(HttpResponse::NotFound().finish());
        }
        return Ok(request.get_error_response())
    };

    if let None = &request.redirect_success {
        return Ok(HttpResponse::Ok()
            .json(UserInfoRequestResponse {
                citizen_id: user.as_ref().unwrap().id,
                username: request.username,
                user_session_token: token.unwrap(),
                info: user.unwrap().get_info().await.unwrap()
            }));
    }
    println!("OK: Redirecting the user");
    Ok(request.get_success_response(token.unwrap()))
}

pub async fn login_simple(pool: web::Data<DBPool>, user: web::Form<UserInfo>) -> Result<HttpResponse, UserAuthError> {
    //TODO: This currently requires a lot of different queries, might perhaps be very slow
    let user_info = user.into_inner();
    let db = pool.get().expect("Unable to get db connection");

    let user = web::block(move || get_user(&db, &user_info))
        .await
        .map_err(|e|UserAuthError::ServerError)?;

    let user = user?;

    let token = web::block(move || insert_new_session(&pool.get().expect("Unable to get db connection"), &user))
        .await
        .map_err(|e|UserAuthError::ServerError)?
        .map_err(|e| UserAuthError::ServerError)?;

    let cookie = Cookie::build("user_session_token", token)
        .domain("supersmartcity.de")
        .secure(true)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .finish())
}
#[derive(Deserialize, Serialize)]

pub struct Token {
    pub code: String
}

//TODO: Proper request version for this should take an redirect uri
pub async fn validate_token_simple(pool: web::Data<DBPool>, token: web::Form<Token>) -> actix_web::Result<HttpResponse, SessionRetrieveError> {

    let check_token_from_request = {
        let code = &token.code;
        let db = pool.get().expect("Unable to get db connection");
        check_token(&db, &code)
    };

    let user = web::block(|| check_token_from_request)
        .await
        .map_err(|_| SessionRetrieveError::ServerError)?;

    if let Err(e) = &user {
        return  Err(SessionRetrieveError::NoSessionFound);
    }
    let user = user?;
    let citizen_info = user.get_info().await.unwrap();

    Ok(HttpResponse::Ok()
        .json(UserInfoRequestResponse {
            citizen_id: user.id,
            username: user.username,
            user_session_token: token.code.clone(),
            info: citizen_info
        }))
}


pub async fn login_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open(PathBuf::from(r"static_content/login_example.html")).unwrap())
}

#[derive(Deserialize, Serialize)]
pub struct EmployeeRegisterRequest {
    pub code: String,

    #[serde(flatten)]
    pub info: NewEmployeeInfo,

    #[serde(flatten)]
    pub credentials: UserInfo

}

pub async fn employee_register(pool: web::Data<DBPool>, data: web::Form<EmployeeRegisterRequest>) -> actix_web::Result<HttpResponse, UserAuthError >{
    let data = data.into_inner();
    let db = pool.get().map_err(|_| UserAuthError::ServerError)?;

    if !(&data.code == "ROOT") {
        let user_verification = move || {
            verify_employee(&db, &data.code.clone()).map_err(|_| UserAuthError::UserNotFound)
        };

        let (session, login_data) = web::block(user_verification)
            .await
            .map_err(|_|UserAuthError::ServerError)??;
        println!("{:?}, {:?}", session, login_data);
    }

    let user_creation = move || {
        let db = pool.clone().get()
            .map_err(|_| UserAuthError::ServerError)?;
        create_employee(&db, &data.info, &data.credentials).map_err(|_| UserAuthError::ServerError)
    };

    web::block(user_creation).await.map_err(|_| UserAuthError::ServerError)??;

    Ok(HttpResponse::Ok().finish())
}


#[derive(Deserialize, Serialize)]
pub struct EmployeeLoginRequestRespone {
    id: u64,
    username: String,
    pub employee_session_token: String,
    pub info: NewEmployeeInfo,

}


pub async fn employee_login(pool: web::Data<DBPool>, credentials: web::Form<UserLoginRequest>) -> Result<HttpResponse, UserAuthError> {
    let credentials = credentials.into_inner();
    let db = pool.get().map_err(|_| UserAuthError::ServerError)?;

    let employee_credentials = UserInfo {
        username: credentials.username,
        password: credentials.password
    };

    let (employee, session) = match web::block(move || login_employee(&db, &employee_credentials))
        .await
        .map_err(|_| UserAuthError::ServerError)? {
        Ok((e,s)) => {(e,s)}
        Err(_) => {
            if let Some(url) = credentials.redirect_error {
                return Ok(HttpResponse::Found()
                    .append_header((LOCATION, HeaderValue::try_from(url).unwrap())).finish());
            }
            return Err(UserAuthError::UserNotFound);
        }
    };


    let e_username = employee.username.clone();
    let e_id = employee.id.clone();
    let get_info = move || {
        let db = &pool.get()
            .map_err(|_| UserAuthError::ServerError)?;
        get_employee_info(&db, &employee).map_err(|_| UserAuthError::UserNotFound)
    };

    let info = web::block(get_info)
        .await
        .map_err(|_| UserAuthError::ServerError)??;

    let response = EmployeeLoginRequestRespone {
        id: e_id,
        username: e_username,
        employee_session_token: session.token,
        info: NewEmployeeInfo {
            firstname: info.firstname,
            lastname: info.lastname,
        }
    };
    let cookie = Cookie::build("employee_session_token", response.employee_session_token.clone())
        .domain("supersmartcity.de")
        .finish();

    if let Some(url) = credentials.redirect_success {
        let http_response = HttpResponse::Found()
            .append_header((LOCATION, HeaderValue::try_from(format!("{}?token={}", url, &response.employee_session_token)).unwrap()))
            .cookie(cookie.clone())
            .finish();
        return Ok(http_response)
    }
    Ok(HttpResponse::Ok().cookie(cookie).json(response))
}

pub async fn employee_verify(pool: web::Data<DBPool>, token: web::Form<Token>) -> Result<HttpResponse, UserAuthError> {
    let db = pool.get().map_err(|_| UserAuthError::ServerError)?;
    let token = token.into_inner().code;
    let (session, employee) = web::block(move || verify_employee(&db, &token)).await.map_err(|_| UserAuthError::ServerError)?.map_err(|_| UserAuthError::UserNotFound)?;

    let e_username = employee.username.clone();
    let e_id = employee.id.clone();
    let get_info = move || {
        let db = &pool.get()
            .map_err(|_| UserAuthError::ServerError)?;
        get_employee_info(&db, &employee).map_err(|_| UserAuthError::UserNotFound)
    };

    let info = web::block(get_info)
        .await
        .map_err(|_| UserAuthError::ServerError)??;

    let response = EmployeeLoginRequestRespone {
        id: e_id,
        username: e_username,
        employee_session_token: session.token,
        info: NewEmployeeInfo {
            firstname: info.firstname,
            lastname: info.lastname,
        }
    };

    Ok(HttpResponse::Ok().json(response))

}