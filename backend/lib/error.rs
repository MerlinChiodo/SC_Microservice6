use std::fmt;
use actix_web::ResponseError;

use moon::actix_web::{error, HttpResponse};
use moon::actix_web::http::StatusCode;
use derive_more::*;
use serde::Deserialize;
use moon::actix_web::http::header::ContentType;
use crate::request::RegistrationRequest;

#[derive(Debug)]
pub enum RegistrationRequestError {
    ServerError,
    InvalidCitizenToken,
    InsertionError(RegistrationRequest),
    DuplicateUserError(RegistrationRequest)
}

impl fmt::Display for RegistrationRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to create user")
    }
}
impl ResponseError for RegistrationRequestError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).finish()
    }
}


#[derive(Deserialize, Debug, Display)]
pub enum AuthErrorType {
    #[display(fmt = "invalid_request")]
    InvalidRequest,
    #[display(fmt = "invalid_client")]
    InvalidClient,
    #[display(fmt = "invalid_grant")]
    InvalidGrant,
    #[display(fmt = "unauthorized_client")]
    UnauthorizedClient,
    #[display(fmt = "unsupported_grant_type")]
    UnsupportedGrantType,
    #[display(fmt = "invalid_scope")]
    InvalidScope
}

#[derive(Error, Deserialize)]
pub struct AuthError {
    pub(crate) error_type: AuthErrorType,
    pub(crate) description: String,
    pub(crate) error_uri: Option<String>
}

impl fmt::Debug for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}


impl error::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}
