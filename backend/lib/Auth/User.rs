use crate::auth::Credentials::{CredentialsHolder, IdentityHolder};
use diesel::dsl::*;
use moon::{chrono, Utc};
use rand::Rng;
use crate::auth::Citizen::IsCitizen;
use crate::auth::Session::Token;
use crate::schema::Sessions::dsl::Sessions;
use crate::schema::Users;
use crate::schema::PendingUsers;

#[derive(Queryable, Identifiable, Clone)]
#[table_name = "Users"]
pub struct User {
    pub id: u64,
    pub username: String,
    pub hash: String
}
impl User {
    pub fn generate_pending_code() -> Token {
        let mut rng = rand::thread_rng();

        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
        //Read max len of code from somewhere maybe
        (0..10)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}
impl IdentityHolder for User {
    fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    fn get_key(&self) -> &str {
        self.username.as_str()
    }
}

impl IsCitizen for User {
    fn get_citizen_id(&self) -> u64 {
        self.id
    }
}
#[derive(Queryable, Identifiable, PartialEq)]
#[table_name="PendingUsers"]
pub struct PendingUser {
    id: u64,
    pub citizen: i64,
    code: String
}


