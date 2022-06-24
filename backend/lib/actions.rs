use std::fmt;
use std::fmt::{Display, format, Formatter, write};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::error::BlockingError;
use diesel::{BoolExpressionMethods, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl};
use diesel_migrations::name;
use diesel::associations;
use rand::{Rng, RngCore};
use crate::models::{EmployeeInfoModel, EmployeeLogin, EmployeeSession, NewEmployeeInfo, NewEmployeeLogin, NewEmployeeSession, Session};
use crate::schema::Sessions::dsl::Sessions;
use crate::schema::Users::dsl::Users;
use crate::schema::Users::{id, username};
use crate::diesel::BelongingToDsl;
use crate::schema::Sessions::{expires, token, user_id};
use diesel::dsl::*;
use diesel::mysql::MysqlQueryBuilder;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use serde_json::Value;
use crate::request::UserRegistrationError;
use crate::schema::EmployeeInfo::dsl::EmployeeInfo;
use crate::schema::EmployeeInfo::{firstname, lastname};
use crate::schema::EmployeeLogins::dsl::EmployeeLogins;
use crate::schema::EmployeeSessions::dsl::EmployeeSessions;
use crate::schema::EmployeeSessions::token as e_token;
use crate::schema::EmployeeLogins::id as ee_id;
use crate::schema::PendingUsers::*;
use crate::schema::PendingUsers::dsl::PendingUsers;
use crate::schema::EmployeeLogins::username as e_username;
use crate::session::{NewSession, SessionCreationError, SessionHolder};
use crate::user::{NewPendingUser, NewUser, PendingUser, User, UserInfo};

#[derive(Debug)]
pub enum UserAuthError {
    DbError(diesel::result::Error),
    VerifyError(SessionCreationError),
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
pub enum SessionRetrieveError {
    DbError(diesel::result::Error),
    NoSessionFound,
    SessionExpired,
    ServerError,
}

impl Display for SessionRetrieveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DbError(err) => write!(f, "Unable to check session due to db error: {:?}", err),
            Self::NoSessionFound => write!(f, "Session can't be found"),
            Self::SessionExpired => write!(f, "The session has expired"),
            _ => write!(f, "Server error")
        }
    }
}
impl ResponseError for SessionRetrieveError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::NOT_FOUND
        }
    }
    fn error_response(&self) -> HttpResponse {HttpResponse::build(self.status_code()).finish()}
}

pub fn insert_new_user(db: &MysqlConnection, user: UserInfo, uid: u64) -> Result<(), UserRegistrationError> {
    let new_user = NewUser::new(uid, &user)
        .map_err(|err| UserRegistrationError::HashError(err))?;


    //TODO: Handle specific insertion errors
    insert_into(Users)
        .values(&new_user)
        .execute(db)
        .map_err(|err| UserRegistrationError::InsertionError(err))?;
    Ok(())
}

pub fn get_user(db: &MysqlConnection, user: &UserInfo) -> Result<User, UserAuthError> {
    let mut results = Users.filter(username.eq(&user.username))
        .limit(1)
        .load::<User>(db)
        .map_err(|err| UserAuthError::DbError(err))?;
    let user_result = results.pop()
        .ok_or(UserAuthError::UserNotFound)?;

    let password_correct = user_result
        .verify(user.password.as_str())
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
pub fn get_token(db: &MysqlConnection, user: &User) -> Result<String, SessionRetrieveError> {
    Ok(get_session(db, user)?.token)
}

pub fn check_token(db: &MysqlConnection, _token: &String) -> Result<User, SessionRetrieveError> {
    //NOTE: 2 Queries right now since diesel only supports now filter with timestamps. Will be changed in the future
    let session: Session = Sessions.filter(token.eq(_token))
        .first(db)
        .map_err(|err| {SessionRetrieveError::DbError(err)})?;
    println!("Executed query");

    if !session.is_valid(){return Err(SessionRetrieveError::SessionExpired)};
    println!("Session is valid");

    Users.filter(id.eq(session.user_id))
        .first(db)
        .map_err(|err| {SessionRetrieveError::DbError(err)})
}
pub fn insert_new_pending_user(db: &MysqlConnection, citizen_id: u64) -> Result<NewPendingUser, diesel::result::Error> {
    let user = NewPendingUser::new(citizen_id);
    diesel::insert_into(PendingUsers)
        .values(&user)
        .execute(db)?;
    Ok(user)
}

pub async fn send_citizen_code(mail_client: &SmtpClient, user: &NewPendingUser) {
    //TODO: Add proper error handling
    let citizen_info = reqwest::get(format!("http://www.smartcityproject.net:9710/api/citizen/{}", user.citizen))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let json_data: Value = serde_json::from_str(&citizen_info).unwrap();

    let mail_adress = json_data.get("email")
        .unwrap()
        .as_str()
        .unwrap();

    let first_name = json_data.get("firstname").unwrap().as_str().unwrap();
    let last_name = json_data.get("lastname").unwrap().as_str().unwrap();
    let name = format!("{} {}", first_name, last_name);
    println!("Sending mail to: {}\n with name {}", mail_adress, name);

    let email = EmailBuilder::new()
        .to(mail_adress)
        .from("support@mail.smartcityproject.net")
        .subject("SmartCity: Ihr Registrierungscode")
        .text(format!("Hallo {}! Ihr persönlicher Registrierungscode lautet: {}", name, user.code))
        .build()
        .unwrap();
    let mut mailer = mail_client.clone().transport();
    let result = mailer.send(email.into());
    println!("Result: {:?}", result);

}

pub fn check_pending_user_token(db: &MysqlConnection, _token: &str) -> Result<PendingUser, diesel::result::Error> {
    let pending_user: PendingUser = PendingUsers.filter(code.eq(_token))
        .first(db)?;

    Ok(pending_user)
}

pub fn create_employee(db: &MysqlConnection, personal_data: &NewEmployeeInfo, credentials: &UserInfo) -> Result<(), UserRegistrationError> {
    insert_into(EmployeeInfo)
        .values(personal_data)
        .execute(db)
        .map_err(|err| UserRegistrationError::InsertionError(err))?;

    let mut employees = EmployeeInfo
        .filter(firstname.eq(&personal_data.firstname).and(lastname.eq(&personal_data.lastname)))
        .limit(1)
        .load::<EmployeeInfoModel>(db)
        .map_err(|err| UserRegistrationError::InsertionError(err))?;

    let employee_result = employees.pop().ok_or(UserRegistrationError::UnknownError)?;

    let secret = NewUser::new(0, credentials).map_err(|e| UserRegistrationError::HashError(e))?;
    let new_employee = NewEmployeeLogin {
        info_id: employee_result.id,
        username: credentials.username.clone(),
        hash: secret.hash.clone()
    };
    insert_into(EmployeeLogins)
        .values(&new_employee)
        .execute(db)
        .map_err(|err| UserRegistrationError::InsertionError(err))?;

    Ok(())
}

pub fn login_employee(db: &MysqlConnection, credentials: &UserInfo) -> Result<(EmployeeLogin, NewEmployeeSession), UserAuthError> {
    let mut results = EmployeeLogins.filter(e_username.eq(&credentials.username))
        .limit(1)
        .load::<EmployeeLogin>(db)
        .map_err(|err| UserAuthError::DbError(err))?;

    let emp_result = results
        .pop()
        .ok_or(UserAuthError::UserNotFound)?;

    emp_result.verify(credentials.password.as_str())
        .map_err(|e| UserAuthError::VerifyError(e))?
        .then(|| true)
        .ok_or(UserAuthError::WrongPassword)?;

    let session: Result<EmployeeSession, diesel::result::Error>= EmployeeSession::belonging_to(&emp_result).first(db);

    if let Ok(s)= session {
        return Ok((emp_result, NewEmployeeSession {
            e_id: s.e_id,
            token: s.token,
            expires: s.expires
        }));
    }

    let session = NewSession::new(&emp_result);
    let emp_session = NewEmployeeSession {
        e_id: emp_result.id,
        token: session.token,
        expires: session.expires };

    insert_into(EmployeeSessions)
        .values(&emp_session)
        .execute(db)
        .map_err(|err| UserAuthError::DbError(err))?;

    Ok((emp_result, emp_session))
}

pub fn verify_employee(db: &MysqlConnection, _token: &str) -> Result<(EmployeeSession, EmployeeLogin), SessionRetrieveError> {
    let session: EmployeeSession = EmployeeSessions.filter(e_token.eq(_token))
        .first(db)
        .map_err(|err| SessionRetrieveError::DbError(err))?;

    session.is_valid().then(|| true).ok_or(SessionRetrieveError::NoSessionFound)?;

    let employee = EmployeeLogins.filter(ee_id.eq(session.e_id))
        .first(db)
        .map_err(|err| SessionRetrieveError::DbError(err))?;

    Ok((session, employee))
}

pub fn get_employee_info(db: &MysqlConnection, employee: &EmployeeLogin) -> Result<EmployeeInfoModel, SessionRetrieveError> {
    EmployeeInfo
        .filter(crate::schema::EmployeeInfo::id.eq(employee.info_id))
        .first(db)
        .map_err(|err| SessionRetrieveError::DbError(err))
}