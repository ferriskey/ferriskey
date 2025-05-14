use chrono::{TimeZone, Utc};
use sea_orm::DatabaseConnection;

use crate::domain::role::entities::models::Role;

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
