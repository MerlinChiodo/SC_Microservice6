use serde::{Serialize, Deserialize};
use crate::auth::Citizen::CitizenInfo;
use crate::auth::Credentials::CredentialsPair;
use crate::auth::Session::Token;
use crate::auth::User::User;
use crate::auth::Employee::{EmployeeLogin, NewEmployeeInfo};

#[derive(Deserialize, Debug)]
pub struct UserRegistrationRequest {
    #[serde(flatten)]
    pub credentials: CredentialsPair,
    pub mail: String,
    pub code: Token,

    pub redirect_success: Option<String>,
    pub redirect_error: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct UserLoginRequest {
    #[serde(flatten)]
    pub credentials: CredentialsPair,

    pub redirect_success: Option<String>,
    pub redirect_error: Option<String>,
}

pub struct UserLoginRequestResponse {
    pub user: User,
    pub new_session_token: Token
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserInfoRequestResponse {
    pub(crate) citizen_id: u64,
    pub(crate) username: String,
    pub(crate) user_session_token: String,
    pub(crate) info: CitizenInfo
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TokenValidateRequest {
    pub code: Token
}

#[derive(Deserialize)]
pub struct EmployeeRegisterRequest {
    pub code: Token,
    #[serde(flatten)]
    pub info: NewEmployeeInfo,

    #[serde(flatten)]
    pub credentials: CredentialsPair,
}

pub struct EmployeeLoginRequestResponse {
    pub employee: EmployeeLogin,
    pub new_employee_token: Token
}

#[derive(Deserialize, Debug)]
pub struct ExternalUserLoginRequest {
    pub redirect_success: Option<String>,
    pub redirect_error: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct EmployeeInfoRequestResponse {
    pub(crate) id: u64,
    pub(crate) username: String,
    pub employee_session_token: Token,
    pub info: NewEmployeeInfo
}