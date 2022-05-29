use std::fmt;
use std::fmt::Formatter;
use base64::DecodeError;
use diesel_migrations::name;
use rand::{Error, RngCore};
use crate::schema::Users;


#[derive(Queryable)]
pub struct User {
    id: u64,
    username: String,
    hash: String,
}
impl User {
    pub fn verify_with_password(&self, password: &str) -> Result<bool, argon2::Error> {
        argon2::verify_encoded(self.hash.as_str(), password.as_bytes())
    }
}

#[derive(Insertable)]
#[table_name="Users"]
pub struct NewUser {
    pub username: String,
    pub hash: String,
}

impl TryFrom<UserInfo> for NewUser {
    type Error = argon2::Error;

    fn try_from(user: UserInfo) -> Result<Self, Self::Error>{
        let mut rng = rand::thread_rng();
        let mut salt = vec![0; 128];

        rng.try_fill_bytes(&mut salt).unwrap();

        let mut config = argon2::Config::default();
        config.hash_length = 128;

        let hash = argon2::hash_encoded(user.password.as_ref(), &salt, &config)?;

        Ok(Self {
            username: user.name,
            hash,
        })
    }
}
pub struct UserInfo {
    pub name: String,
    pub password: String,
}
impl fmt::Display for UserInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let b64 = format!("{}:{}", self.name, self.password);
        let result = base64::encode(&b64);
        write!(f, "{}", result)
    }
}

//TODO: Add error handling if string is malformed
impl From<String> for UserInfo {
    fn from(string: String) -> Self {
        let result:Vec<&str> = string.split(':').collect();

        Self {
            name: result[0].parse().unwrap(),
            password: result[1].parse().unwrap()
        }
    }
}