use ferriskey_core::domain::role::value_objects::CreateRoleRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRoleValidator {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleValidator {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRolePermissionsValidator {
    pub permissions: Vec<String>,
}

impl CreateRoleValidator {
    pub fn to_dto(self, realm_id: Uuid, client_id: Option<Uuid>) -> CreateRoleRequest {
        CreateRoleRequest {
            name: self.name,
            description: self.description,
            permissions: self.permissions,
            realm_id,
            client_id,
        }
    }
}
