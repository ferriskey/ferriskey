use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateClientValidator {
    pub client_id: String,
    pub name: String,
    pub client_type: String,
    pub service_account_enabled: bool,
    pub public_client: bool,
    pub protocol: String,
    pub enabled: bool,
}
