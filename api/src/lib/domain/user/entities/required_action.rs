use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
pub enum RequiredAction {
    ConfigureOtp,
    VerifyEmail,
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
