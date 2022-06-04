use actix_web::{HttpResponse, Responder, web};
use actix_web::cookie::Cookie;
use actix_web::error::BlockingError;
use actix_web::error::Kind::Http;
use crate::actions::{check_token, get_session, get_user, insert_new_session, insert_new_user, SessionRetrieveError, UserAuthError, UserRegistrationError};
use crate::models::{Token, User, UserIdentityInfo, UserLoginRequest, UserRegisterRequest};
use crate::server::DBPool;
use actix_web::post;
use crate::error::RegistrationRequestError;
use crate::schema::Users::username;

pub async fn register(pool: web::Data<DBPool>,
                      request: web::Form<UserRegisterRequest>) -> Result<HttpResponse, RegistrationRequestError> {
    let db = pool.get()
        .map_err(|_|RegistrationRequestError::ServerError)?;
    let request = request.into_inner();

    let user_identity = UserIdentityInfo {
        name: request.username.clone(),
        password: request.password.clone()
    };

    let result = web::block(move || insert_new_user(&db, user_identity))
        .await
        .map_err(|_| RegistrationRequestError::ServerError)?;

    //TODO: Maybe let the client handle the redirect in the future
    match result {
        Ok(_) => Ok(request.get_success_response()),
        Err(_) => Ok(request.get_error_response())
    }
}

pub async fn login(pool: web::Data<DBPool>, request: web::Form<UserLoginRequest>) -> Result<HttpResponse, UserAuthError> {

    let request = request.into_inner();
    let user_info = UserIdentityInfo {
        name: request.username.clone(),
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
    let db2 = pool.get().map_err(|err| UserAuthError::ServerError)?;

    let token = web::block(move || insert_new_session(&db2,&user.unwrap()))
        .await
        .map_err(|e| UserAuthError::ServerError)?;

    if let Err(e) = token{
        return Ok(request.get_error_response())
    };

    Ok(request.get_success_response(token.unwrap()))
}

//TODO: Ask for url instead of just returning a cookie
//NOTE: THIS IS HORRIBLE
pub async fn login_simple(pool: web::Data<DBPool>, user: web::Form<UserIdentityInfo>) -> Result<HttpResponse, UserAuthError> {
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

    Ok(HttpResponse::Ok()
        .cookie(Cookie::new("user_session_token", token))
        .finish())
}

//TODO: Proper request version for this should take an redirect uri
pub async fn validate_token_simple(pool: web::Data<DBPool>, token: web::Form<Token>) -> Result<HttpResponse, SessionRetrieveError> {
    let db = pool.get().expect("Unable to get db connection");

    let user = web::block(move || check_token(&db, &token.code))
        .await
        .map_err(|_| SessionRetrieveError::ServerError)?;

    let user = user?;
    Ok(HttpResponse::Ok().json((user.id, user.username)))
}
