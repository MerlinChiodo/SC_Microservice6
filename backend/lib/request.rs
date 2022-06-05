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
    pub redirect_success: String,
    pub redirect_error: String
}

impl std::fmt::Display for RegistrationRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl Request for RegistrationRequest {
    fn get_success_response(&self) -> HttpResponse {
        HttpResponse::Ok()
            .status(StatusCode::FOUND)
            .append_header((LOCATION, HeaderValue::try_from(&self.redirect_success).unwrap()))
            .finish()
    }

    fn get_error_response(&self) -> HttpResponse {
        HttpResponse::Ok()
            .status(StatusCode::FOUND)
            .append_header((LOCATION, HeaderValue::try_from(&self.redirect_error).unwrap()))
            .finish()
    }
}
