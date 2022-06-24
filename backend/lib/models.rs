use std::fmt;
use std::fmt::{Display, Formatter};
use actix_web::{HttpResponse, Responder};
use actix_web::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use actix_web::http::header::LOCATION;
use actix_web::web::Header;
use base64::DecodeError;
use diesel_migrations::name;
use moon::{chrono, Duration, Utc};
use rand::{Error, Rng, RngCore};
use rand::distributions::Distribution;
use crate::schema::Users;
use crate::schema::Sessions;

use crate::schema::EmployeeLogins;
use crate::schema::EmployeeInfo;
use crate::schema::PendingUsers;
use crate::schema::EmployeeSessions;

use serde::{Deserialize, Serialize};
use crate::schema::Users::username;
use crate::session::{SessionCreationError, SessionHolder};
use crate::user::{User, UserInfo};
/*NOTE: The definition of a session or a session token may change in the future.
    However this should not affect any api calls. To the user, a token may always be interpreted
    as an opaque key.
 */
#[derive(Queryable, Identifiable, PartialEq, Associations)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name="Sessions"]
pub struct Session {
    pub(crate) id: u64,
    pub(crate) user_id: u64,
    pub token: String,
    expires: chrono::NaiveDateTime
}

impl Session {
    pub fn is_valid(&self) -> bool {
        self.expires >= Utc::now().naive_utc()
    }
}
#[derive(Deserialize, Debug)]
pub struct ExternalUserLoginRequest {
    pub redirect_success: Option<String>,
    pub redirect_error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UserLoginRequest {
    pub username: String,
    pub password: String,

    pub redirect_success: Option<String>,
    pub redirect_error: Option<String>,
}

impl Display for UserLoginRequest{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl UserLoginRequest {
    pub fn get_success_response(&self, token: String) -> HttpResponse {
        let cookie = actix_web::cookie::Cookie::build("user_session_token", token.clone())
            .domain("smartcityproject.net")
            .finish();

        if let Some(redirect) = &self.redirect_success {
            HttpResponse::Found()
                .append_header((LOCATION, HeaderValue::try_from(format!("{}?token={}", redirect, token)).unwrap()))
                .cookie(cookie)
                .finish()
        } else {
            HttpResponse::Found()
                .cookie(cookie)
                .finish()
        }
    }

    //TODO: Add error info
    pub fn get_error_response(&self) -> HttpResponse {
        if let Some(redirect) = &self.redirect_error {
            HttpResponse::Found()
                .append_header((LOCATION, HeaderValue::try_from(redirect).unwrap()))
                .finish()
        } else {
            HttpResponse::Forbidden().finish()
        }
    }
}
#[derive(Queryable, Identifiable, PartialEq, Associations)]
#[table_name="EmployeeInfo"]
pub struct EmployeeInfoModel {
    pub id: u64,
    pub firstname: String,
    pub lastname: String
}

#[derive(Deserialize, Serialize, Insertable, PartialEq, Associations)]
#[table_name="EmployeeInfo"]
pub struct NewEmployeeInfo {
    pub firstname: String,
    pub lastname: String
}

#[derive(Queryable, Identifiable, PartialEq, Associations, Clone)]
#[belongs_to(EmployeeInfoModel, foreign_key = "info_id")]
#[table_name="EmployeeLogins"]
pub struct EmployeeLogin {
    pub id: u64,
    pub info_id: u64,
    pub username: String,
    pub hash: String
}
impl SessionHolder for EmployeeLogin {
    fn verify(&self, secret: &str) -> Result<bool, SessionCreationError> {
        argon2::verify_encoded(self.hash.as_str(), secret.as_bytes()).map_err(|e| SessionCreationError::HashError(e))
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

#[derive(Insertable, PartialEq, Associations)]
#[table_name="EmployeeLogins"]
pub struct NewEmployeeLogin {
    pub(crate) info_id: u64,
    pub(crate) username: String,
    pub(crate) hash: String
}

#[derive(Queryable, Identifiable, PartialEq, Associations)]
#[belongs_to(EmployeeLogin, foreign_key ="e_id")]
#[table_name="EmployeeSessions"]
pub struct EmployeeSession {
    pub id: u64,
    pub e_id: u64,
    pub token: String,
    pub expires: chrono::NaiveDateTime
}
impl EmployeeSession {
    pub fn is_valid(&self) -> bool {
        self.expires >= Utc::now().naive_utc()
    }
}

#[derive(Insertable, PartialEq, Associations)]
#[table_name="EmployeeSessions"]
pub struct NewEmployeeSession {
    pub(crate) e_id: u64,
    pub(crate) token: String,
    pub(crate) expires: chrono::NaiveDateTime
}

//TODO: Maybe sign the token or later include additional stuff