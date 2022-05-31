use std::fmt;
use std::fmt::{Display, Formatter, write};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use diesel::{ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl};
use diesel_migrations::name;
use diesel::associations;
use rand::{Rng, RngCore};
use crate::models::{NewSession, NewUser, Session, User, UserInfo};
use crate::schema::Sessions::dsl::Sessions;
use crate::schema::Users::dsl::Users;
use crate::schema::Users::{id, username};
use crate::diesel::BelongingToDsl;
use crate::schema::Sessions::{expires, token, user_id};
use diesel::dsl::*;

#[derive(Debug)]
pub enum UserRegistrationError {
    HashError(argon2::Error),
    InsertionError(diesel::result::Error)
}

impl fmt::Display for UserRegistrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserRegistrationError::HashError(e) => {
                write!(f, "HashError: {}", e)
            }
            UserRegistrationError::InsertionError(e) => {
                write!(f, "InsertionError: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum UserAuthError {
    DbError(diesel::result::Error),
    VerifyError(argon2::Error),
    ServerError,
    UserNotFound,
    WrongPassword
}

impl Display for UserAuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserAuthError::UserNotFound => write!(f,"The user does not exist"),
            UserAuthError::WrongPassword => write!(f, "The provided password does not match the username"),
            _ => write!(f, "Internal error")
        }
    }
}
impl ResponseError for UserAuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::WrongPassword => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).finish()
    }
}

#[derive(Debug)]
pub enum SessionCreationError {
    DbError(diesel::result::Error),
}

impl Display for SessionCreationError {
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
#[derive(Debug)]
pub enum SessionRetrieveError {
    DbError(diesel::result::Error),
    NoSessionFound,
    SessionExpired,
}

pub fn insert_new_user(db: &MysqlConnection, user: UserInfo) -> Result<(), UserRegistrationError> {
    let new_user = NewUser::try_from(user)
        .map_err(|err| UserRegistrationError::HashError(err))?;


    //TODO: Handle specific insertion errors
    diesel::insert_into(Users)
        .values(&new_user)
        .execute(db)
        .map_err(|err| UserRegistrationError::InsertionError(err))?;
    Ok(())
}

pub fn get_user(db: &MysqlConnection, user: &UserInfo) -> Result<User, UserAuthError> {
    let mut results = Users.filter(username.eq(&user.name))
        .limit(1)
        .load::<User>(db)
        .map_err(|err| UserAuthError::DbError(err))?;
    let user_result = results.pop()
        .ok_or(UserAuthError::UserNotFound)?;

    let password_correct = user_result
        .verify_with_password(user.password.as_str())
        .map_err(|err| UserAuthError::VerifyError(err))?;

    if password_correct {Ok(user_result)} else {Err(UserAuthError::WrongPassword)}
}

pub fn insert_new_session(db: &MysqlConnection, user: &User) -> Result<String, SessionCreationError> {
    //TODO: Invalidate old sessions or maybe don't do anything if a session already exists
    let session = NewSession::new(user);

    diesel::insert_into(Sessions)
        .values(&session)
        .execute(db)
        .map_err(|err| SessionCreationError::DbError(err))?;
    Ok((session.token))
}

pub fn get_session(db: &MysqlConnection, user: &User) -> Result<Session, SessionRetrieveError> {
    //TODO: Handle cases where entry was not found differently
    let session: Session = Session::belonging_to(user)
        .first(db)
        .map_err(|err| {SessionRetrieveError::DbError(err)})?;

    if session.is_valid(){(Ok(session))} else {Err(SessionRetrieveError::SessionExpired)}
}

pub fn check_token(db: &MysqlConnection, _token: String) -> Result<User, SessionRetrieveError> {
    //NOTE: 2 Queries right now since diesel only supports now filter with timestamps. Will be changed in the future

    let session: Session = Sessions.filter(token.eq(_token))
        .first(db)
        .map_err(|err| {SessionRetrieveError::DbError(err)})?;

    if !session.is_valid(){return Err(SessionRetrieveError::SessionExpired)};

    Users.filter(id.eq(session.user_id))
        .first(db)
        .map_err(|err| {SessionRetrieveError::DbError(err)})
}



