use actix_web::{HttpResponse, Responder, web};
use actix_web::cookie::Cookie;
use actix_web::error::BlockingError;
use actix_web::error::Kind::Http;
use crate::actions::{check_token, get_session, get_user, insert_new_session, insert_new_user, SessionRetrieveError, UserAuthError, UserRegistrationError};
use crate::models::{Token, User, UserIdentityInfo};
use crate::server::DBPool;
use actix_web::post;

//TODO: Use base64 encoding
pub async fn register_simple(pool: web::Data<DBPool>, new_user: web::Form<UserIdentityInfo>) -> impl Responder {
    let db = pool.get().expect("Unable to get db connection");
    let result = web::block(move || insert_new_user(&db, new_user.into_inner()))
        .await;
    match result {
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(_) => {
            HttpResponse::InternalServerError().finish()
        }
    }
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
        .map_err(|e|UserAuthError::ServerError)? //TODO: Add proper error handling
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
