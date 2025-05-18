use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::{
    role::{
        entities::{CreateRoleDto, errors::RoleError, models::Role},
        ports::RoleRepository,
    },
    utils::generate_uuid_v7,
};

impl From<entity::roles::Model> for Role {
    fn from(model: entity::roles::Model) -> Self {
        Role {
            id: model.id,
            name: model.name,
            description: model.description,

            permissions: model.permissions,
            realm_id: model.realm_id,
            client_id: model.client_id,
            client: None,
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        }
    }
}

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
    async fn create(&self, payload: CreateRoleDto) -> Result<Role, RoleError> {
        let id = generate_uuid_v7();
        let model = entity::roles::ActiveModel {
            id: Set(id),
            name: Set(payload.name),
            description: Set(payload.description),
            permissions: Set(payload.permissions as i64),
            realm_id: Set(payload.realm_id),
            client_id: Set(payload.client_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        Ok(result.into())
    }

    async fn delete_by_id(&self, id: uuid::Uuid) -> Result<(), RoleError> {
        let result = entity::roles::Entity::delete_many()
            .filter(entity::roles::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        if result.rows_affected == 0 {
            return Err(RoleError::InternalServerError);
        }

        Ok(())
    }

    async fn get_by_client_id(&self, client_id: uuid::Uuid) -> Result<Vec<Role>, RoleError> {
        let roles = entity::roles::Entity::find()
            .filter(entity::roles::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .iter()
            .map(|model| model.clone().into())
            .collect::<Vec<Role>>();

        Ok(roles)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Role>, RoleError> {
        let role = entity::roles::Entity::find()
            .filter(entity::roles::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .map(Role::from);

        Ok(role)
    }
}
