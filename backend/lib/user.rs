use actix_web::http::header::LOCATION;
use actix_web::http::{HeaderValue, StatusCode};
use actix_web::{HttpResponse, Resource};
use rand::{Rng, RngCore};
use serde::Deserialize;
use crate::session::{SessionCreationError, SessionHolder};
use diesel::dsl::*;
use diesel::MysqlConnection;
use crate::schema::Users;
use crate::schema::PendingUsers;
use crate::server::DBPool;

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
    pub id: u64,
    pub username: String,
    pub hash: String,
}
impl NewUser {
    pub fn new(id: u64, info: &impl ResourceOwnerCredentials) -> Result<Self, argon2::Error> {
        let mut rng = rand::thread_rng();
        let mut salt = vec![0; 128];

        rng.try_fill_bytes(&mut salt).unwrap();

        let mut config = argon2::Config::default();
        config.hash_length = 128;

        let hash = argon2::hash_encoded(info.get_secret().as_bytes(), &salt, &config)?;

        Ok(Self {
            id,
            username: String::from(info.get_key()),
            hash,
        })
    }
}

#[derive(Queryable, Identifiable, PartialEq)]
#[table_name="PendingUsers"]
pub struct PendingUser {
    id: u64,
    pub citizen: i64,
    code: String
}

#[derive(Insertable)]
#[table_name="PendingUsers"]
pub struct NewPendingUser {
    citizen: i64,
    code: String
}

impl NewPendingUser {
    pub fn new(citizen_id: u64) -> Self {
        let mut rng = rand::thread_rng();

        let mut code = rand::thread_rng();
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
        //Read max len of code from somewhere maybe
        let code = (0..10)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        Self {
            citizen: citizen_id as i64,
            code
        }
    }
}