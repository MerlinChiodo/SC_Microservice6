use anyhow::ensure;
use diesel::mysql::Mysql;
use diesel::{BoolExpressionMethods, ExpressionMethods, insert_into, MysqlConnection, QueryDsl, QueryResult, RunQueryDsl};
use diesel::result::Error;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use moon::Utc;
use crate::auth::Credentials::{CredentialsHolder, CredentialsPair, IdentityHolder};
use thiserror::Error;
use crate::auth::Citizen::{Citizen, CitizenInfo};
use crate::auth::Employee::{EmployeeInfoModel, EmployeeLogin, EmployeeSession, NewEmployeeInfo};
use crate::auth::Errors::{AuthenticationError, AuthenticationResult, DatabaseError, LoginError, LoginResult, SessionInsertionError, SessionInsertionResult, SessionRetrievalError, SessionRetrievalResult, UserRegistrationError, UserRegistrationResult};
use crate::auth::Request::{UserRegistrationRequest, UserLoginRequest, UserLoginRequestResponse, EmployeeLoginRequestResponse};
use crate::auth::Session::{NewSession, Session, Token, UserSession};
use crate::auth::User::{PendingUser, User};
use crate::schema;
use crate::schema::EmployeeInfo::dsl::EmployeeInfo;
use crate::schema::EmployeeLogins::dsl::EmployeeLogins;
use crate::schema::EmployeeSessions::dsl::EmployeeSessions;
use crate::schema::PendingUsers::dsl::PendingUsers;
use crate::schema::Sessions::dsl::Sessions;
use crate::schema::Sessions::{expires, token};
use crate::schema::Users::dsl::Users;
use crate::schema::Users::{hash, id, username};

pub struct Actions;

fn insert_new_user(db: &MysqlConnection, credentials: &impl CredentialsHolder, uid: u64) -> UserRegistrationResult<()> {
    let salt = credentials.create_hash()?;

    insert_into(Users)
        .values((id.eq(&uid),
                  username.eq(credentials.get_key()),
                  hash.eq(&salt)))

        .execute(db)
        .map_err(|e| UserRegistrationError::Db(e.into()))?;

    Ok(())
}

fn authenticate_user(db: &MysqlConnection, credentials: &impl CredentialsHolder) -> AuthenticationResult<User> {
    let mut results = Users.filter(username.eq(credentials.get_key()))
        .load::<User>(db)
        .map_err(|e| AuthenticationError::Db(e.into()))?;

    let user_result = results.pop()
        .ok_or(AuthenticationError::UserNotFound)?;

    user_result.verify(credentials)?
        .then(|| user_result)
        .ok_or(AuthenticationError::WrongPassword)
}

fn insert_user_session(db: &MysqlConnection, user: &User) -> SessionInsertionResult<NewSession> {
    use crate::schema::Sessions::{expires, token, user_id};

    let session = NewSession::new()?;
    insert_into(Sessions)
        .values((user_id.eq(&user.id), token.eq(&session.token), expires.eq(&session.expires)))
        .execute(db)
        .map_err(|e| SessionInsertionError::Db(e.into()))
        .map(|_| session)

}

fn get_user_session(db: &MysqlConnection, user: &User) -> SessionRetrievalResult<UserSession> {
    use crate::diesel::BelongingToDsl;

    let session: UserSession = UserSession::belonging_to(user)
        .first(db)
        .map_err(|err| SessionRetrievalError::Db(err.into()))?;

    session
        .is_valid()
        .then(|| session)
        .ok_or(SessionRetrievalError::InvalidSession)
}

pub fn check_user_session_token(db: &MysqlConnection, _token: &Token) -> SessionRetrievalResult<User> {
    use crate::schema::Sessions::{expires, token, user_id};

    let session: UserSession = Sessions.filter(token.eq(_token))
        .first(db)
        .map_err(|err| SessionRetrievalError::Db(err.into()))?;

    session
        .is_valid()
        .then(|| true)
        .ok_or(SessionRetrievalError::InvalidSession)?;

    Users.filter(id.eq(session.user_id))
        .first(db)
        .map_err(|err| SessionRetrievalError::Db(err.into()))
}

pub fn insert_new_pending_user(db: &MysqlConnection, citizen_id: i64) -> Result<Token, DatabaseError> {
    use crate::schema::PendingUsers::{citizen, code};

    let pending_code = User::generate_pending_code();

    insert_into(PendingUsers)
        .values((citizen.eq(&citizen_id), code.eq(&pending_code)))
        .execute(db)?;

    Ok(pending_code)
}
pub fn check_pending_user_token(db: &MysqlConnection, _token: &str) -> Result<PendingUser, diesel::result::Error> {
    use crate::schema::PendingUsers::code;
    let pending_user: PendingUser = PendingUsers.filter(code.eq(_token))
        .first(db)?;

    Ok(pending_user)
}

pub fn register_user(db: &MysqlConnection, request: &UserRegistrationRequest) -> UserRegistrationResult<()> {
    let pending_user = check_pending_user_token(db, &request.code)
        .map_err(|err| UserRegistrationError::Db(err.into()))?;

    //TODO: Remove pending user
    insert_new_user(db, &request.credentials, pending_user.citizen as u64)
}

pub fn login_user(db: &MysqlConnection, request: &UserLoginRequest) -> LoginResult<UserLoginRequestResponse> {
    let user = authenticate_user(db, &request.credentials)?;

    let user_token = get_user_session(db, &user)
        .map_or_else(|_| insert_user_session(db, &user).map_err(|e| LoginError::SessionInsertion(e)), |s| Ok(NewSession { token: s.token, expires: s.expires }))?;
    
    Ok(UserLoginRequestResponse{ user, new_session_token: user_token.token})
}

pub async fn send_citizen_code(mail_client: &SmtpClient, citizen: &CitizenInfo, code: &Token) -> anyhow::Result<()>{
    ensure!(citizen.email.is_some());
    let mail_adress = citizen.email.clone().unwrap();
    let name = format!("{} {}", citizen.firstname, citizen.lastname);
    println!("Sending mail to: {}\n with name {}", mail_adress, name);

    let email = EmailBuilder::new()
        .to(mail_adress)
        .from("support@mail.smartcityproject.net")
        .subject("SmartCity: Ihr Registrierungscode")
        .text(format!("Hallo {}! Ihr persönlicher Registrierungscode lautet: {}. Registrieren Sie sich unter: http://www.supersmartcity.de:9760", name, code))
        .build()?;

    let mut mailer = mail_client.clone().transport();
    let result = mailer.send(email.into());
    println!("Result: {:?}", result);
    Ok(())
}

pub fn register_employee(db: &MysqlConnection, employee_data: &NewEmployeeInfo, credentials: &CredentialsPair) -> UserRegistrationResult<()> {
    use crate::schema::EmployeeInfo::{firstname, lastname};
    use crate::schema::EmployeeInfo::dsl::EmployeeInfo;
    use crate::schema::EmployeeLogins::{info_id, username, hash};

    insert_into(EmployeeInfo)
        .values(employee_data)
        .execute(db)
        .map_err(|err| UserRegistrationError::Db(err.into()))?;

    let mut employees = EmployeeInfo
        .filter(firstname.eq(&employee_data.firstname).and(lastname.eq(&employee_data.lastname)))
        .limit(1)
        .load::<EmployeeInfoModel>(db)
        .map_err(|err| UserRegistrationError::Db(err.into()))?;

    let employee_result = employees.pop().ok_or(UserRegistrationError::DataRetrieval)?;

    let new_hash = credentials.create_hash()?;
    insert_into(EmployeeLogins)
        .values((info_id.eq(&employee_result.id), username.eq(credentials.get_key()), hash.eq(&new_hash)))
        .execute(db)
        .map_err(|err| UserRegistrationError::Db(err.into()))?;

    Ok(())
}

pub fn login_employee(db: &MysqlConnection, credentials: &CredentialsPair) -> LoginResult<EmployeeLoginRequestResponse> {
    use schema::EmployeeLogins::{username};
    use schema::EmployeeSessions;
    use schema::EmployeeSessions::{e_id, token, expires};
    use crate::diesel::BelongingToDsl;
    let mut results = EmployeeLogins.filter(username.eq(credentials.get_key()))
        .limit(1)
        .load::<EmployeeLogin>(db)
        .map_err(|e| LoginError::Db(e.into()))?;

    let emp_result: EmployeeLogin = results
        .pop()
        .ok_or(LoginError::Authentication(AuthenticationError::UserNotFound))?;

    emp_result.verify(credentials)
        .map_err(|e| LoginError::Authentication(AuthenticationError::Verification(e)))?
        .then(|| true).ok_or(LoginError::Authentication(AuthenticationError::UserNotFound))?;


    let sessions_result: Result<Vec<EmployeeSession>, diesel::result::Error>= EmployeeSession::belonging_to(&emp_result)
        .load(db);

    if let Ok(sessions) = sessions_result {
        let session_count = sessions.len();
        if session_count > 0 {
            let s = &sessions[session_count - 1];
            println!("Found a session...");
            if s.is_valid(){
                println!("Session is valid, returning");
                return Ok(EmployeeLoginRequestResponse {
                    employee: emp_result.clone(),
                    new_employee_token: s.token.clone()
                });
            }

            println!("Expiration: {:?} current: {:?}", s.expires, Utc::now().naive_utc());
            println!("Session is invalid, returning a new one");
            //TODO: Remove invalid session
        }
    }
    let session = NewSession::new()?;
    insert_into(EmployeeSessions)
        .values((e_id.eq(&emp_result.id), token.eq(&session.token), expires.eq(&session.expires)))
        .execute(db)
        .map_err(|e| LoginError::Db(e.into()))?;

    Ok(EmployeeLoginRequestResponse{
        employee: emp_result,
        new_employee_token: session.token
    })
}

pub fn verify_employee(db: &MysqlConnection, _token: &Token) -> SessionRetrievalResult<EmployeeLoginRequestResponse> {
    use schema::EmployeeSessions;
    use schema::EmployeeSessions::{e_id, token, expires};
    use schema::EmployeeLogins;
    use schema::EmployeeLogins::{id};

    let session: EmployeeSession = EmployeeSessions.filter(token.eq(_token))
        .first(db)
        .map_err(|err| SessionRetrievalError::Db(err.into()))?;

    let employee = EmployeeLogins.filter(id.eq(&session.e_id))
        .first(db)
        .map_err(|e| SessionRetrievalError::Db(e.into()))?;

    session.is_valid().then(|| {
        EmployeeLoginRequestResponse {
            employee,
            new_employee_token: session.token
        }
    }).ok_or(SessionRetrievalError::InvalidSession)
}

pub fn get_employee_info(db: &MysqlConnection, employee: &EmployeeLogin) -> SessionRetrievalResult<EmployeeInfoModel> {
    use schema::EmployeeInfo;

    EmployeeInfo
        .filter(crate::schema::EmployeeInfo::id.eq(employee.info_id))
        .first(db)
        .map_err(|err| SessionRetrievalError::Db(err.into()))
}
