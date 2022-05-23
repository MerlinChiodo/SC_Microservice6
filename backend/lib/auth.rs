use std::char::ParseCharError;
use moon::actix_web::http::Uri;
use moon::actix_web::{Either, HttpResponse, web};
use secrecy::Secret;
use serde::Deserialize;
use crate::error::{AuthError, AuthErrorType};

pub trait RequestData<'a>: Deserialize<'a> {
    fn validate(&self) -> Result<(), AuthError>;
}

#[derive(Deserialize)]
pub struct AuthClientInfo {
    id: String,
    secret: String
}

impl RequestData<'_> for AuthClientInfo {
    fn validate(&self) -> Result<(), AuthError> {
        if self.id.len() >= 512 {
            return Err(AuthError {
                error_type: AuthErrorType::InvalidRequest,
                description: "Client identifier is too long".to_string(),
                error_uri: None
            })
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AuthorizationRequest {
    response_type: String,
    client_info: AuthClientInfo,
    redirect_uri: String,
    permissions: String,
    state: Option<String>,
    requires_internal: bool,
    owner_secret: String
}

impl RequestData<'_> for AuthorizationRequest {
    fn validate(&self) -> Result<(), AuthError> {
        self.client_info.validate()?;
        if self.response_type != "code" {
            return Err(AuthError {
                error_type: AuthErrorType::InvalidRequest,
                description: "Invalid response type argument".to_string(),
                error_uri: None
            })
        }
        Ok(())
        //TODO: Check uri validity
        //TODO: Check permssions
        //TODO: Check owner_secret
    }
}
#[derive(Deserialize)]
pub struct AuthResponse {
    code: String,
    state: String
}

type AuthRequestResponse = Either<HttpResponse, Result<AuthResponse,moon::actix_web::Error>>;

/*
#[post("/")]
async fn auth(request: web::Form<AuthorizationRequest>) -> AuthRequestResponse {
}
*/