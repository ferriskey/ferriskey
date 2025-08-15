use ferriskey_core::domain::authentication::entities::GrantType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct TokenRequestValidator {
    #[serde(default)]
    pub grant_type: GrantType,

    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,

    #[serde(default)]
    pub client_secret: Option<String>,

    #[serde(default)]
    pub code: Option<String>,

    #[serde(default)]
    pub username: Option<String>,

    #[serde(default)]
    pub password: Option<String>,

    #[serde(default)]
    pub refresh_token: Option<String>,
}
