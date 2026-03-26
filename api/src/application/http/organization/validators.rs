use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateOrganizationValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(length(min = 1, message = "alias is required"))]
    pub alias: String,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateOrganizationValidator {
    #[validate(length(min = 1, message = "name must not be empty when provided"))]
    pub name: Option<String>,
    #[validate(length(min = 1, message = "alias must not be empty when provided"))]
    pub alias: Option<String>,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

fn default_enabled() -> bool {
    true
}
