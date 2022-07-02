use std::hash::Hash;
use anyhow::Result;
use async_trait::async_trait;
use diesel::Identifiable;
use serde::{Serialize, Deserialize};
use crate::auth::Errors::CitizenInfoRetrievalResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitizenAddress {
    pub street: Option<String>,
    pub housenumber: Option<String>,
    pub city_code: Option<u32>,
    pub city: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CitizenInfo {
    pub firstname: String,
    pub lastname: String,
    pub gender: Option<String>,
    pub birthdate: Option<String>,
    pub place_of_birth: Option<String>,
    pub birthname: Option<String>,
    pub email: Option<String>,
    pub spouse_id: Option<u64>,
    pub child_ids: Option<Vec<u64>>,

    pub address: CitizenAddress
}


#[async_trait]
pub trait IsCitizen {
    fn get_citizen_id(&self) -> u64;

    async fn get_citizen_info(&self) -> CitizenInfoRetrievalResult<CitizenInfo> {
        let user_info = reqwest::get(format!("http://www.smartcityproject.net:9710/api/citizen/{}", self.get_citizen_id()))
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&user_info)?)
    }
}

pub struct Citizen {
    pub citizen_id: u64
}

impl IsCitizen for Citizen {
    fn get_citizen_id(&self) -> u64 {
        self.citizen_id
    }
}
