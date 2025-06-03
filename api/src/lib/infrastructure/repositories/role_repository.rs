use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::{
    role::{
        entities::{CreateRoleDto, errors::RoleError, models::Role, permission::Permissions},
        ports::RoleRepository,
    },
    utils::generate_uuid_v7,
};

impl From<entity::roles::Model> for Role {
    fn from(model: entity::roles::Model) -> Self {
        let permissions = Permissions::from_bitfield(model.permissions as u64);
        let permissions = permissions
            .iter()
            .map(|p| p.name().to_string())
            .collect::<Vec<String>>();

        Role {
            id: model.id,
            name: model.name,
            description: model.description,
            permissions,
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
        let permissions = Permissions::from_names(&payload.permissions);
        let bitfield = Permissions::to_bitfield(&permissions);

        let model = entity::roles::ActiveModel {
            id: Set(id),
            name: Set(payload.name),
            description: Set(payload.description),
            permissions: Set(bitfield as i64),
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

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<Role>, RoleError> {
        let roles = entity::roles::Entity::find()
            .filter(entity::roles::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|_| RoleError::NotFound)?;

        let roles: Vec<Role> = roles.into_iter().map(|role| role.into()).collect();

        Ok(roles)
    }

    async fn find_by_name(&self, name: String, realm_id: Uuid) -> Result<Option<Role>, RoleError> {
        let role = entity::roles::Entity::find()
            .filter(entity::roles::Column::Name.eq(name))
            .filter(entity::roles::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .map(Role::from);

        Ok(role)
    }
}
