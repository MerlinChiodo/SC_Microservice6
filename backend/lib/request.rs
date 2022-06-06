use actix_web::error::Kind::Http;
use actix_web::http::{HeaderValue, StatusCode};
use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use crate::user::UserInfo;
use serde::Deserialize;

pub trait Request {
    fn get_success_response(&self) -> HttpResponse;
    fn get_error_response(&self) -> HttpResponse;
}

#[derive(Deserialize, Debug)]
pub struct RegistrationRequest {
    #[serde(flatten)]
    pub info: UserInfo,
    pub mail: String,
    pub code: String,

    pub redirect_success: Option<String>,
    pub redirect_error: Option<String>
}

impl std::fmt::Display for RegistrationRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl Request for RegistrationRequest {
    fn get_success_response(&self) -> HttpResponse {
        if let Some(redirect) = &self.redirect_success {
            HttpResponse::Ok()
                .status(StatusCode::FOUND)
                .append_header((LOCATION, HeaderValue::try_from(redirect).unwrap()))
                .finish()
        } else {
            HttpResponse::Ok()
                .status(StatusCode::FOUND)
                .append_header((LOCATION, HeaderValue::try_from("/page/login").unwrap()))
                .finish()
        }

    }

    fn get_error_response(&self) -> HttpResponse {
        if let Some(redirect) = &self.redirect_error {
            HttpResponse::Ok()
                .status(StatusCode::FOUND)
                .append_header((LOCATION, HeaderValue::try_from(redirect).unwrap()))
                .finish()
        } else {
            HttpResponse::Forbidden().finish()
        }
    }
}
