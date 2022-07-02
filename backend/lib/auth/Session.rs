use std::fmt;
use std::fmt::{Formatter, write};
use std::hash::Hash;
use std::io::Write;
use moon::{chrono, NaiveDateTime, Utc};
use anyhow::{ensure, Result};
use diesel::{ExpressionMethods, Identifiable, Insertable, MysqlConnection, QueryDsl, RunQueryDsl, Table};
use diesel::backend::Backend;
use diesel::serialize::Output;
use diesel::sql_types::Text;
use diesel::types::{ToSql, VarChar};
use rand::Rng;
use crate::schema::Sessions;
use crate::schema::Sessions::user_id;
use derive_more::Display;
use diesel::deserialize::FromSql;
use crate::auth::Errors::SessionCreationError;
use crate::auth::User::User;
pub type Token = String;
fn create_token() -> Token {
        let mut rng = rand::thread_rng();
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

        (0..64).map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
}

pub trait Session {
    fn expires(&self) -> &NaiveDateTime;
    fn token(&self) -> &Token;

    fn is_valid(&self) -> bool {
        self.expires() >= &Utc::now().naive_utc()
    }
}

pub struct NewSession {
    pub token: Token,
    pub expires: NaiveDateTime
}
impl NewSession {
    pub fn new() -> Result<Self, SessionCreationError> {
        let token = create_token();
        Utc::now()
            .naive_utc()
            .checked_add_signed(chrono::Duration::days(1))
            .ok_or(SessionCreationError::Overflow)
            .map(|e| Self {token, expires: e})
    }
}
impl Session for NewSession {
    fn expires(&self) -> &NaiveDateTime {
        &self.expires
    }

    fn token(&self) -> &Token {
        &self.token
    }
}

#[derive(Queryable, Identifiable, PartialEq, Associations)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name="Sessions"]
pub struct UserSession {
    pub(crate) id: u64,
    pub(crate) user_id: u64,
    pub token: Token,
    pub(crate) expires: NaiveDateTime
}

impl Session for UserSession {
    fn expires(&self) -> &NaiveDateTime {
        &self.expires
    }

    fn token(&self) -> &Token {
        &self.token
    }
}
