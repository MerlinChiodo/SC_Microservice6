use rand::RngCore;
use crate::session::{SessionCreationError, SessionHolder};
use crate::user::CitizenAddress;
use serde::{Serialize, Deserialize};

/*
#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
#[table_name = "Employees"]
pub struct Employee {
    pub id: u64,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub mail: String,
    hash: String,
}

impl SessionHolder for Employee {
    fn verify(&self, secret: &str) -> Result<bool, SessionCreationError> {
        argon2::verify_encoded(self.hash.as_str(), secret.as_bytes()).map_err(|e| SessionCreationError(e))
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

#[derive(Insertable)]
#[table_name="Employees"]
pub struct NewEmployee {
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub mail: String,
    hash: String,
}

impl NewEmployee {
    pub fn new(username: String, first_name: String, last_name: String, mail: String) -> Result<Self, argon2::Error> {
        let mut rng = rand::thread_rng();
        let mut salt = vec![0; 128];

        rng.try_fill_bytes(&mut salt).unwrap();

        let mut config = argon2::Config::default();
        config.hash_length = 128;

        let hash = argon2::hash_encoded(info.get_secret().as_bytes(), &salt, &config)?;

        Ok(Self {
            username,
            first_name,
            last_name,
            mail,
            hash
        })
    }
}
*/

