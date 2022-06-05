use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use moon::{chrono, Utc};
use rand::Rng;

#[derive(Debug)]
pub enum SessionCreationError {
    DbError(diesel::result::Error),
    HashError(argon2::Error)
}

impl fmt::Display for SessionCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to create session")
    }
}

impl ResponseError for SessionCreationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}

pub trait SessionHolder {
    fn verify(&self, secret: &str) -> Result<bool, SessionCreationError>;
    fn get_id(&self) -> u64;
}

pub struct NewSession {
    user_id: u64,
    token: String,
    expires: chrono::NaiveDateTime
}

impl NewSession {
    pub fn new(holder: impl SessionHolder) -> Self {
        let mut rng = rand::thread_rng();
        //TODO: This doesn't adhere to oauth2 std
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
        let token = (0..64)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        //TODO: This is stupid. In the future we should use timestamps or SQL specific stuff
        //TODO: Read this stuff from config file maybe

        //TODO: Nothing bad should happen, however we might want to add error handling anyways
        let expires = Utc::now()
            .naive_utc()
            .checked_add_signed(chrono::Duration::days(1))
            .expect("Unable to create session");

        //TODO: This feels unsafe, maybe we should not pass the user id like this
        NewSession {
            user_id: holder.get_id(),
            token,
            expires
        }
    }

}

