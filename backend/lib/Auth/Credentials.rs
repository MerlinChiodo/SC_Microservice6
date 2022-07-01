use anyhow::Result;
use diesel::Insertable;
use rand::{CryptoRng, RngCore};
use crate::auth::Errors::{CredentialsCreationError, CredentialsCreationResult, CredentialsVerificationError, CredentialsVerificationResult};
use serde::Deserialize;

pub trait CredentialsHolder {
    fn get_secret(&self) -> &str;
    fn get_key(&self) -> &str;

    fn create_hash(&self) -> CredentialsCreationResult<String> {
        let mut rng = rand::thread_rng();
        let mut salt = vec![0; 128];
        rng.try_fill_bytes(&mut salt)?;

        let mut config = argon2::Config::default();
        config.hash_length = 128;
        Ok(argon2::hash_encoded(self.get_secret().as_bytes(), &salt, &config)?)
    }
}

#[derive(Deserialize, Debug)]
pub struct CredentialsPair {
    username: String,
    password: String
}

impl CredentialsHolder for CredentialsPair {
    fn get_secret(&self) -> &str {
        self.password.as_str()
    }
    fn get_key(&self) -> &str {
        self.username.as_str()
    }
}

struct IdentityPair(String, String);

pub trait IdentityHolder {
    fn get_hash(&self) -> &str;
    fn get_key(&self) -> &str;

    fn verify(&self, other: &impl CredentialsHolder) -> CredentialsVerificationResult<bool> {
        Ok(argon2::verify_encoded(self.get_hash(), other.get_secret().as_bytes())?)
    }
}

impl IdentityHolder for IdentityPair {
    fn get_hash(&self) -> &str {
        self.1.as_str()
    }

    fn get_key(&self) -> &str {
        self.0.as_str()
    }
}
