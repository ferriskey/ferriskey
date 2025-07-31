use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{common::generate_uuid_v7, realm::entities::Realm, role::entities::Role};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
#[typeshare]
pub struct User {
    #[typeshare(serialized_as = "string")]
    pub id: Uuid,
    #[typeshare(serialized_as = "string")]
    pub realm_id: Uuid,
    #[typeshare(serialized_as = "string")]
    pub client_id: Option<Uuid>,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
    pub roles: Vec<Role>,
    pub realm: Option<Realm>,
    pub required_actions: Vec<RequiredAction>,
    #[typeshare(serialized_as = "Date")]
    pub created_at: DateTime<Utc>,
    #[typeshare(serialized_as = "Date")]
    pub updated_at: DateTime<Utc>,
}

pub struct UserConfig {
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("User already exists")]
    AlreadyExists,
    #[error("Invalid user")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,

    #[error("Forbidden: {0}")]
    Forbidden(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[typeshare]
pub enum RequiredAction {
    #[serde(rename = "configure_otp")]
    ConfigureOtp,
    #[serde(rename = "verify_email")]
    VerifyEmail,
    #[serde(rename = "update_password")]
    UpdatePassword,
}

impl Display for RequiredAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredAction::ConfigureOtp => write!(f, "configure_otp"),
            RequiredAction::VerifyEmail => write!(f, "verify_email"),
            RequiredAction::UpdatePassword => write!(f, "update_password"),
        }
    }
}

impl TryFrom<String> for RequiredAction {
    type Error = RequiredActionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "configure_otp" => Ok(RequiredAction::ConfigureOtp),
            "verify_email" => Ok(RequiredAction::VerifyEmail),
            "update_password" => Ok(RequiredAction::UpdatePassword),
            _ => Err(RequiredActionError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum RequiredActionError {
    #[error("Required action not found")]
    NotFound,
    #[error("Required action already exists")]
    AlreadyExists,
    #[error("Invalid required action")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,
}

impl User {
    pub fn new(user_config: UserConfig) -> Self {
        let now = Utc::now();
        let id = generate_uuid_v7();

        Self {
            id,
            realm_id: user_config.realm_id,
            client_id: user_config.client_id,
            username: user_config.username,
            firstname: user_config.firstname,
            lastname: user_config.lastname,
            email: user_config.email,
            email_verified: user_config.email_verified,
            enabled: user_config.enabled,
            roles: Vec::new(),
            realm: None,
            required_actions: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
