use crate::auth::Credentials::{CredentialsHolder, IdentityHolder};
use serde::{Deserialize, Serialize};
use diesel::{Insertable, Identifiable, Queryable, Associations};
use moon::{chrono, NaiveDateTime};
use crate::auth::Errors::SessionCreationError;
use crate::auth::Session::{Session, Token};
use crate::schema::{EmployeeInfo, EmployeeLogins, EmployeeSessions};
#[derive(Queryable, Identifiable, PartialEq, Associations)]
#[table_name="EmployeeInfo"]
pub struct EmployeeInfoModel {
    pub id: u64,
    pub firstname: String,
    pub lastname: String
}

#[derive(Deserialize, Serialize, Insertable, PartialEq, Associations, Clone)]
#[table_name="EmployeeInfo"]
pub struct NewEmployeeInfo {
    pub firstname: String,
    pub lastname: String
}

#[derive(Queryable, Identifiable, PartialEq, Associations, Clone, Debug)]
#[belongs_to(EmployeeInfoModel, foreign_key = "info_id")]
#[table_name="EmployeeLogins"]
pub struct EmployeeLogin {
    pub id: u64,
    pub info_id: u64,
    pub username: String,
    pub hash: String
}

impl IdentityHolder for EmployeeLogin {
    fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    fn get_key(&self) -> &str {
        self.username.as_str()
    }
}

#[derive(Queryable, Identifiable, PartialEq, Associations, Debug)]
#[belongs_to(EmployeeLogin, foreign_key ="e_id")]
#[table_name="EmployeeSessions"]
pub struct EmployeeSession {
    pub id: u64,
    pub e_id: u64,
    pub token: String,
    pub expires: chrono::NaiveDateTime
}

impl Session for EmployeeSession {
    fn expires(&self) -> &NaiveDateTime {
        &self.expires
    }

    fn token(&self) -> &Token {
        &self.token
    }
}