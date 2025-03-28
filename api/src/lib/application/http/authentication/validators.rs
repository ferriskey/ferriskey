use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::authentication::entities::model::GrantType;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct TokenRequestValidator {
    pub grant_type: GrantType,

    #[validate(length(min = 1, message = "client_id is required"))]
    pub client_id: String,

    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}
