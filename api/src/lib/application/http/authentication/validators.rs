use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct TokenRequestValidator {
    #[validate(length(min = 1, message = "grant_type is required"))]
    pub grant_type: String,
    
    #[validate(length(min = 1, message = "client_id is required"))]
    pub client_id: String,
    
    #[validate(length(min = 1, message = "code is required"))]
    pub code: String,
}
