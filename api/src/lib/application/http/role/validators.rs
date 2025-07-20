use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domain::role::entities::CreateRoleDto;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[typeshare]
pub struct CreateRoleValidator {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[typeshare]
pub struct UpdateRoleValidator {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[typeshare]
pub struct UpdateRolePermissionsValidator {
    pub permissions: Vec<String>,
}

impl CreateRoleValidator {
    pub fn to_dto(self, realm_id: Uuid, client_id: Option<Uuid>) -> CreateRoleDto {
        CreateRoleDto {
            name: self.name,
            description: self.description,
            permissions: self.permissions,
            realm_id,
            client_id,
        }
    }
}
