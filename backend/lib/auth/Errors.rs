use std::fmt::{Display, Formatter};
use actix_web::{error, HttpRequest, HttpResponse, ResponseError};
use actix_web::error::{QueryPayloadError, UrlencodedError};
use actix_web::http::StatusCode;
use diesel::r2d2;
use diesel::result::{DatabaseErrorKind, Error};
use serde_json::json;
use thiserror::Error;


pub trait IntoHttpError<T> {
    fn http_error(self, message: &str, status_code: StatusCode) -> Result<T, actix_web::Error>;

    fn http_internal_error(self, message: &str) -> Result<T, actix_web::Error>
    where Self: Sized {
        self.http_error(message, StatusCode::INTERNAL_SERVER_ERROR)
    }

}

impl <T, E: std::fmt::Debug> IntoHttpError<T> for Result<T, E> {
    fn http_error(self, message: &str, status_code: StatusCode) -> Result<T, actix_web::Error> {

        self.map_err(|e| error::InternalError::new(message.to_string(), status_code ).into())
    }
}

pub fn query_error_handler(err: QueryPayloadError, _: &HttpRequest) -> actix_web::Error{
    error::InternalError::from_response("",
                                        HttpResponse::BadRequest()
                                            .content_type("application/json")
                                            .body(format!(r#"{{"type": "InvalidRequest", "error": "{:?}"}}"#, err))).into()
}


pub fn form_error_handler(err: UrlencodedError, _: &HttpRequest) -> actix_web::Error {
    error::InternalError::from_response(
        "",
        HttpResponse::BadRequest()
            .content_type("application/json")
            .body(format!(r#"{{"type": "InvalidRequest", "error": "{:?}"}}"#, err)),
    ).into()
}

pub type CredentialsCreationResult<T> =  Result<T, CredentialsCreationError>;
pub type CredentialsVerificationResult<T> = Result<T, CredentialsVerificationError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection to the database failed")]
    Connection,
    #[error("Duplicate entry where duplicate entries are not allowed")]
    Duplicate,
    #[error("Entry not found")]
    NotFound,
    #[error("Contraint error")]
    Constraint,
    #[error("Unknown Database error")]
    Other(diesel::result::Error)
}

impl ResponseError for DatabaseError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(json!({"type": "database", "error": &self.to_string()}))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {

        match err {
            Error::DatabaseError(t, _) => {
                match t {
                    DatabaseErrorKind::UniqueViolation => Self::Duplicate,
                    DatabaseErrorKind::ForeignKeyViolation => Self::Constraint,
                    _ => DatabaseError::Other(err)
                }
            },
            _ => Self::Other(err)
        }
    }
}
pub type CitizenInfoRetrievalResult<T> = Result<T, CitizenInfoRetrievalError>;

#[derive(Error, Debug)]
pub enum CitizenInfoRetrievalError {
    #[error("Unable to get information from remote server")]
    Request(#[from] reqwest::Error),

    #[error("Unable to parse citizen info")]
    Parse(#[from] serde_json::error::Error)
}

#[derive(Error, Debug)]
pub enum CredentialsCreationError {
    #[error("Unable to create salt")]
    RNG(#[from] rand::Error),

    #[error("Unable to create hash")]
    Hash(#[from]argon2::Error)
}

#[derive(Error, Debug)]
pub enum CredentialsVerificationError {
    #[error("Unable to verify hash")]
    Verify(#[from] argon2::Error),
}

pub type UserRegistrationResult<T> = Result<T, UserRegistrationError>;
#[derive(Error, Debug)]
pub enum UserRegistrationError {
    #[error("Database issue")]
    Db(#[from] DatabaseError),

    #[error("Unable to create user")]
    UserCreation(#[from] CredentialsCreationError),

    #[error("Citizen Code could not be found")]
    InvalidCitizenCode,

    #[error("Connection issue")]
    Connection(#[from] actix_web::error::BlockingError),

    #[error("Unable to retrieve data")]
    DataRetrieval,

    #[error("Unable to authenticate")]
    Auth(#[from] SessionRetrievalError),
}

impl ResponseError for UserRegistrationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(json!({"type": "user_registration", "error": &self.to_string()}))
    }
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::Db(e) => e.status_code(),
            Self::InvalidCitizenCode => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub type AuthenticationResult<T> = Result<T, AuthenticationError>;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Database issue")]
    Db(#[from] DatabaseError),

    #[error("User was not found")]
    UserNotFound,

    #[error("Unable to verify the user")]
    Verification(#[from] CredentialsVerificationError),

    #[error("The provided password is wrong")]
    WrongPassword
}

#[derive(Error, Debug)]
pub enum SessionCreationError {
    #[error("Overflow Error")]
    Overflow
}

pub type SessionInsertionResult<T> = Result<T, SessionInsertionError>;
#[derive(Error, Debug)]
pub enum SessionInsertionError {
    #[error("Database issue")]
    Db(#[from] DatabaseError),

    #[error("Unable to create session")]
    Creation(#[from] SessionCreationError),
}

#[derive(Error, Debug)]
pub enum SessionRetrievalError {
    #[error("Database issue")]
    Db(#[from] DatabaseError),

    #[error("Session is invalid")]
    InvalidSession,

    #[error("Connection issue")]
    Connection(#[from] actix_web::error::BlockingError),

    #[error("Unable to retrieve citizen info")]
    Info(#[from] CitizenInfoRetrievalError)
}
impl ResponseError for SessionRetrievalError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(json!({"type": "session_retrieval", "error": &self.to_string()}))
    }
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::Db(e) => e.status_code(),
            Self::InvalidSession => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
pub type SessionRetrievalResult<T> = Result<T, SessionRetrievalError>;
pub type LoginResult<T> = Result<T, LoginError>;
#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Database issue")]
    Db(#[from] DatabaseError),

    #[error("Connection issue")]
    Connection(#[from] actix_web::error::BlockingError),

    #[error("Unable to create new session")]
    SessionCreation(#[from] SessionCreationError),
    #[error("Unable to authenticate user")]
    Authentication(#[from] AuthenticationError),

    #[error("Unable to insert new session")]
    SessionInsertion(#[from] SessionInsertionError),

    #[error("Unable to retrieve citizen info")]
    Info(#[from] CitizenInfoRetrievalError),

    #[error("Unable to retrieve citizen info")]
    SessionRetrieval(#[from] SessionRetrievalError)
}

impl ResponseError for LoginError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(json!({"type": "login", "error": &self.to_string()}))
    }
    fn status_code(&self) -> StatusCode {
        match &self {
            LoginError::Db(e) => e.status_code(),
            LoginError::Connection(_) => StatusCode::INTERNAL_SERVER_ERROR,
            LoginError::Authentication(_) => StatusCode::FORBIDDEN,
            LoginError::SessionInsertion(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
#[derive(Error, Debug)]
pub enum MailSenderError {

}