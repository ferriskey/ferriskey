use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum GrantType {
    #[default]
    #[serde(rename = "authorization_code")]
    Code,

    #[serde(rename = "password")]
    Password,

    #[serde(rename = "client_credentials")]
    Credentials,

    #[serde(rename = "refresh_token")]
    RefreshToken,
}

impl Display for GrantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrantType::Code => write!(f, "code"),
            GrantType::Password => write!(f, "password"),
            GrantType::Credentials => write!(f, "credentials"),
            GrantType::RefreshToken => write!(f, "refresh_token"),
        }
    }
}
