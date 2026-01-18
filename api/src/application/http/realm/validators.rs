use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRealmValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRealmValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRealmSettingValidator {
    pub default_signing_algorithm: Option<String>,

    pub user_registration_enabled: Option<bool>,
    pub forgot_password_enabled: Option<bool>,
    pub remember_me_enabled: Option<bool>,
    pub magic_link_enabled: Option<bool>,
    pub magic_link_ttl_minutes: Option<u32>,
}
