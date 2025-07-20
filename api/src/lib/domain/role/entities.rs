use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;

pub mod errors;
pub mod models;
pub mod permission;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare]
pub struct CreateRoleDto {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    #[typeshare(serialized_as = "string")]
    pub realm_id: Uuid,
    #[typeshare(serialized_as = "string", optional)]
    pub client_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRoleDto {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRolePermissionsDto {
    pub permissions: Vec<String>,
}
