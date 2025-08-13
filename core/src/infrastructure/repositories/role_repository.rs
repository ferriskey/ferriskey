use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::{
    common::generate_uuid_v7,
    role::{
        entities::{Role, RoleError, permission::Permissions},
        ports::RoleRepository,
        value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
};

impl From<crate::entity::roles::Model> for Role {
    fn from(model: crate::entity::roles::Model) -> Self {
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
    async fn create(&self, payload: CreateRoleRequest) -> Result<Role, RoleError> {
        let id = generate_uuid_v7();
        let permissions = Permissions::from_names(&payload.permissions);
        let bitfield = Permissions::to_bitfield(&permissions);

        let model = crate::entity::roles::ActiveModel {
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

    async fn get_by_client_id(&self, client_id: uuid::Uuid) -> Result<Vec<Role>, RoleError> {
        let roles = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .iter()
            .map(|model| model.clone().into())
            .collect::<Vec<Role>>();

        Ok(roles)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Role>, RoleError> {
        let roles_with_clients = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Id.eq(id))
            .find_with_related(crate::entity::clients::Entity)
            .all(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        if roles_with_clients.is_empty() {
            return Ok(None);
        }

        let (role_model, related_clients) = &roles_with_clients[0];
        let mut role: Role = role_model.clone().into();

        // Only set client if it exists
        if let Some(client_model) = related_clients.first() {
            role.client = Some(client_model.clone().into());
        }

        Ok(Some(role))
    }

    async fn delete_by_id(&self, id: uuid::Uuid) -> Result<(), RoleError> {
        let result = crate::entity::roles::Entity::delete_many()
            .filter(crate::entity::roles::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        if result.rows_affected == 0 {
            return Err(RoleError::InternalServerError);
        }

        Ok(())
    }

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<Role>, RoleError> {
        let roles = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::RealmId.eq(realm_id))
            .find_with_related(crate::entity::clients::Entity)
            .all(&self.db)
            .await
            .map_err(|_| RoleError::NotFound)?;

        if roles.is_empty() {
            return Ok(Vec::new());
        }

        let roles: Vec<Role> = roles
            .into_iter()
            .map(|(role, clients)| {
                let mut role: Role = role.into();
                if let Some(client) = clients.first() {
                    role.client = Some(client.clone().into());
                }
                role
            })
            .collect();

        Ok(roles)
    }

    async fn find_by_name(&self, name: String, realm_id: Uuid) -> Result<Option<Role>, RoleError> {
        let role = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Name.eq(name))
            .filter(crate::entity::roles::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .map(Role::from);

        Ok(role)
    }

    async fn update_by_id(&self, id: Uuid, payload: UpdateRoleRequest) -> Result<Role, RoleError> {
        let role = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .ok_or(RoleError::NotFound)?;

        let mut role: crate::entity::roles::ActiveModel = role.into();
        if let Some(name) = payload.name {
            role.name = Set(name);
        }

        role.description = Set(payload.description);

        let updated_role: Role = role
            .update(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .into();

        Ok(updated_role)
    }

    async fn update_permissions_by_id(
        &self,
        id: Uuid,
        payload: UpdateRolePermissionsRequest,
    ) -> Result<Role, RoleError> {
        let role = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .ok_or(RoleError::NotFound)?;

        let permissions = Permissions::from_names(&payload.permissions);
        let bitfield = Permissions::to_bitfield(&permissions);

        let mut role: crate::entity::roles::ActiveModel = role.into();
        role.permissions = Set(bitfield as i64);

        let updated_role: Role = role
            .update(&self.db)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .into();

        Ok(updated_role)
    }
}
