use actix_web::http::header::LOCATION;
use actix_web::http::{HeaderValue, StatusCode};
use actix_web::{HttpResponse, Resource};
use rand::RngCore;
use serde::Deserialize;
use crate::session::{SessionCreationError, SessionHolder};
use diesel::dsl::*;
use crate::schema::Users;

pub trait ResourceOwnerCredentials {
    fn get_key(&self) -> &str;
    fn get_secret(&self) -> &str;
}

#[derive(Queryable, Identifiable)]
#[table_name = "Users"]
pub struct User {
    pub(crate) id: u64,
    pub username: String,
    hash: String,
}

impl SessionHolder for User {
    fn verify(&self, secret: &str) -> Result<bool, SessionCreationError> {
        argon2::verify_encoded(self.hash.as_str(), secret.as_bytes()).map_err(|e| SessionCreationError::HashError(e))
    }

    fn get_id(&self) -> u64 {
        self.id
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserInfo {
    pub(crate) username: String,
    pub(crate) password: String
}

impl ResourceOwnerCredentials for UserInfo {
    fn get_key(&self) -> &str {
        self.username.as_str()
    }

    fn get_secret(&self) -> &str {
        self.password.as_str()
    }
}


impl std::fmt::Display for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b64 = format!("{}:{}", self.username, self.password);
        let result = base64::encode(&b64);
        write!(f, "{}", result)
    }
}

impl From<String> for UserInfo {
    fn from(string: String) -> Self {
        let result: Vec<&str> = string.split(':').collect();
        Self {
            username: result[0].parse().unwrap(),
            password: result[1].parse().unwrap()
        }
    }
}

impl UserInfo {
    pub fn verify(&self) -> bool {
        self.username.len() >= 5 &&  self.username.len() <= 255 && self.password.len() >= 10
    }
}
#[derive(Insertable)]
#[table_name="Users"]
pub struct NewUser {
    pub username: String,
    pub hash: String,
}
impl NewUser {
    pub fn new(user: &UserInfo) -> Result<Self, argon2::Error> {
        let mut rng = rand::thread_rng();
        let mut salt = vec![0; 128];

        rng.try_fill_bytes(&mut salt).unwrap();

        let mut config = argon2::Config::default();
        config.hash_length = 128;

        let hash = argon2::hash_encoded(user.password.as_ref(), &salt, &config)?;

        Ok(Self {
            username: user.username.clone(),
            hash,
        })
    }
}
