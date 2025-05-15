use chrono::{TimeZone, Utc};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::domain::role::{
    entities::{CreateRoleRequest, errors::RoleError, models::Role},
    ports::RoleRepository,
};

// impl From<entity::roles::Model> for Role {
//     fn from(model: entity::roles::Model) -> Self {
//         Role {
//             id: model.id,
//             name: model.name,
//             description: model.description,

//             permissions: model.permissions,
//             realm_id: model.realm_id,
//             client_id: model.client_id,
//             created_at: Utc.from_utc_datetime(&model.created_at),
//             updated_at: Utc.from_utc_datetime(&model.updated_at),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct PostgresRoleRepository {
    pub db: DatabaseConnection,
}

impl PostgresRoleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl RoleRepository for PostgresRoleRepository {
    async fn create(&self, payload: CreateRoleRequest) -> Result<(), RoleError> {
        todo!()
    }

    async fn delete_by_id(&self, id: uuid::Uuid) -> Result<(), RoleError> {
        todo!()
    }

    async fn get_by_client_id(&self, client_id: uuid::Uuid) -> Result<Vec<Role>, RoleError> {
        todo!()
    }

    async fn get_by_client_id_text(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> Result<Vec<Role>, RoleError> {
        todo!()
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Role>, RoleError> {
        todo!()
    }
}
