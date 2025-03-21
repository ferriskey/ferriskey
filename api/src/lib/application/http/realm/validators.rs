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
