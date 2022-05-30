use diesel::{ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl};
use diesel_migrations::name;
use diesel::associations;
use rand::{Rng, RngCore};
use crate::models::{NewSession, NewUser, Session, User, UserInfo};
use crate::schema::Sessions::dsl::Sessions;
use crate::schema::Users::dsl::Users;
use crate::schema::Users::username;
use crate::diesel::BelongingToDsl;
#[derive(Debug)]
pub enum UserRegistrationError {
    HashError(argon2::Error),
    InsertionError(diesel::result::Error)
}

#[derive(Debug)]
pub enum UserAuthError {
    DbError(diesel::result::Error),
    VerifyError(argon2::Error),
    UserNotFound,
    WrongPassword
}

#[derive(Debug)]
pub enum SessionCreationError {
    DbError(diesel::result::Error),
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

pub fn insert_new_session(db: &MysqlConnection, user: &User) -> Result<(), SessionCreationError> {
    let session = NewSession::new(user);

    diesel::insert_into(Sessions)
        .values(&session)
        .execute(db)
        .map_err(|err| SessionCreationError::DbError(err))?;
    Ok(())
}

pub fn get_session(db: &MysqlConnection, user: &User) -> Result<Session, SessionRetrieveError> {
    //TODO: Handle cases where entry was not found differently
    let session = Session::belonging_to(user)
        .first(db)
        .map_err(|err| {SessionRetrieveError::DbError(err)})?;

    if session.is_valid{(Ok(session))} else {Err(SessionRetrieveError::SessionExpired)}

}



