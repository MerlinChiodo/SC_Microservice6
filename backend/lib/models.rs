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
use serde::Deserialize;
use crate::schema::Users::username;
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
#[derive(Insertable)]
#[table_name="Sessions"]
pub struct NewSession {
    user_id: u64,
    pub(crate) token: String,
    expires: chrono::NaiveDateTime
}

impl NewSession {
    pub fn new(user: &User) -> Self {
        //TODO: Read size from some config file maybe
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
            .checked_add_signed(Duration::days(1))
            .expect("Unable to create session");

        //TODO: This feels unsafe, maybe we should not pass the user id like this
        NewSession {
            user_id: user.id,
            token,
            expires
        }
    }

}

#[derive(Deserialize)]
pub struct Token {
    pub code: String
}



#[derive(Deserialize, Debug)]
pub struct UserLoginRequest {
    pub username: String,
    pub password: String,

    pub redirect_success: String,
    pub redirect_error: String,
}

impl Display for UserLoginRequest{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl UserLoginRequest {
    pub fn get_success_response(&self, token: String) -> HttpResponse {
        HttpResponse::Ok()
            .status(StatusCode::FOUND)
            .append_header((LOCATION, HeaderValue::try_from(format!("{}/{}", &self.redirect_success, token)).unwrap()))
            .finish()
    }

    //TODO: Add error info
    pub fn get_error_response(&self) -> HttpResponse {
        HttpResponse::Ok()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .append_header((LOCATION, HeaderValue::try_from(&self.redirect_success).unwrap()))
            .finish()
    }
}

//TODO: Maybe sign the token or later include additional stuff