use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryTrait, Select,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::realm::entities::{RealmScope, Scoped, Unscoped};
use crate::domain::{
    common::{
        entities::app_errors::CoreError,
        generate_uuid_v7,
        pagination::{Page, PageRequest},
    },
    role::{
        entities::{Role, RoleFilter, RoleSortField, permission::Permissions},
        ports::RoleRepository,
        value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
};
use crate::entity::{clients, roles};
use crate::infrastructure::pagination::{SortColumn, contains, paginate};

impl SortColumn<roles::Entity> for RoleSortField {
    fn column(&self) -> roles::Column {
        match self {
            RoleSortField::Name => roles::Column::Name,
            RoleSortField::CreatedAt => roles::Column::CreatedAt,
            RoleSortField::UpdatedAt => roles::Column::UpdatedAt,
        }
    }
}

fn listing_select(realm_id: Uuid, filter: &RoleFilter) -> Select<roles::Entity> {
    roles::Entity::find()
        .filter(roles::Column::RealmId.eq(realm_id))
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(roles::Column::Name, value))
        })
        .apply_if(filter.description.as_deref(), |select, value| {
            select.filter(contains(roles::Column::Description, value))
        })
        .apply_if(filter.require_mfa, |select, value| {
            select.filter(roles::Column::RequireMfa.eq(value))
        })
        .apply_if(filter.client_id, |select, client_id| {
            select.filter(roles::Column::ClientId.eq(client_id))
        })
        .apply_if(filter.ids.as_deref(), |select, ids| {
            select.filter(roles::Column::Id.is_in(ids.iter().copied()))
        })
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
    async fn create(&self, payload: CreateRoleRequest) -> Result<Role, CoreError> {
        let id = generate_uuid_v7();
        let permissions = Permissions::from_names(&payload.permissions);
        let bitfield = Permissions::to_bitfield(&permissions);

        let model = crate::entity::roles::ActiveModel {
            id: Set(id),
            name: Set(payload.name),
            description: Set(payload.description),
            permissions: Set(bitfield as i64),
            realm_id: Set(payload.realm_id.into()),
            client_id: Set(payload.client_id),
            require_mfa: Set(false),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(result.into())
    }

    async fn get_by_client_id(&self, client_id: uuid::Uuid) -> Result<Vec<Role>, CoreError> {
        let roles = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .iter()
            .map(|model| model.clone().into())
            .collect::<Vec<Role>>();

        Ok(roles)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Unscoped<Role>>, CoreError> {
        let roles_with_clients = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Id.eq(id))
            .find_with_related(crate::entity::clients::Entity)
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if roles_with_clients.is_empty() {
            return Ok(None);
        }

        let (role_model, related_clients) = &roles_with_clients[0];
        let mut role: Role = role_model.clone().into();

        // Only set client if it exists
        if let Some(client_model) = related_clients.first() {
            role.client = Some(client_model.clone().into());
        }

        Ok(Some(Unscoped::new(role)))
    }

    async fn delete_by_id(&self, role: &Scoped<Role>) -> Result<(), CoreError> {
        let result = crate::entity::roles::Entity::delete_many()
            .filter(crate::entity::roles::Column::Id.eq(role.get().id))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if result.rows_affected == 0 {
            return Err(CoreError::InternalServerError);
        }

        Ok(())
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<RoleFilter, RoleSortField>,
    ) -> Result<Page<Role>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("error listing roles: {:?}", e);
            CoreError::InternalServerError
        })?;

        let client_ids: Vec<Uuid> = models.iter().filter_map(|role| role.client_id).collect();
        let clients: HashMap<Uuid, clients::Model> = if client_ids.is_empty() {
            HashMap::new()
        } else {
            clients::Entity::find()
                .filter(clients::Column::Id.is_in(client_ids))
                .all(&self.db)
                .await
                .map_err(|e| {
                    error!("error loading the clients of listed roles: {:?}", e);
                    CoreError::InternalServerError
                })?
                .into_iter()
                .map(|client| (client.id, client))
                .collect()
        };

        let roles = models
            .into_iter()
            .map(|model| {
                let client = model
                    .client_id
                    .and_then(|id| clients.get(&id).cloned())
                    .map(Into::into);
                let mut role = Role::from(model);
                role.client = client;
                role
            })
            .collect();

        Ok(Page::new(roles, total, request.page, request.limit))
    }

    async fn find_by_name(
        &self,
        name: String,
        realm_id: Uuid,
    ) -> Result<Option<Unscoped<Role>>, CoreError> {
        let role = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Name.eq(name))
            .filter(crate::entity::roles::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .map(Role::from)
            .map(Unscoped::new);

        Ok(role)
    }

    async fn update_by_id(
        &self,
        role: &Scoped<Role>,
        payload: UpdateRoleRequest,
    ) -> Result<Role, CoreError> {
        let role = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Id.eq(role.get().id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::NotFound)?;

        let mut role: crate::entity::roles::ActiveModel = role.into();
        if let Some(name) = payload.name {
            role.name = Set(name);
        }

        role.description = Set(payload.description);

        if let Some(require_mfa) = payload.require_mfa {
            role.require_mfa = Set(require_mfa);
        }

        let updated_role: Role = role
            .update(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .into();

        Ok(updated_role)
    }

    async fn update_permissions_by_id(
        &self,
        role: &Scoped<Role>,
        payload: UpdateRolePermissionsRequest,
    ) -> Result<Role, CoreError> {
        let role = crate::entity::roles::Entity::find()
            .filter(crate::entity::roles::Column::Id.eq(role.get().id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::NotFound)?;

        let permissions = Permissions::from_names(&payload.permissions);
        let bitfield = Permissions::to_bitfield(&permissions);

        let mut role: crate::entity::roles::ActiveModel = role.into();
        role.permissions = Set(bitfield as i64);

        let updated_role: Role = role
            .update(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .into();

        Ok(updated_role)
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::role::entities::RoleFilter;

    fn sql(filter: &RoleFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&RoleFilter::default());
        assert!(
            sql.contains(r#""roles"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches() {
        let sql = sql(&RoleFilter {
            name: Some("a%".to_string()),
            description: Some("b".to_string()),
            ..RoleFilter::default()
        });
        assert!(sql.contains(r#""roles"."name" ILIKE E'%a\\%%'"#), "{sql}");
        assert!(
            sql.contains(r#""roles"."description" ILIKE '%b%'"#),
            "{sql}"
        );
    }

    #[test]
    fn exact_filters_are_equalities() {
        let client = Uuid::from_u128(7);
        let sql = sql(&RoleFilter {
            require_mfa: Some(true),
            client_id: Some(client),
            ..RoleFilter::default()
        });
        assert!(sql.contains(r#""roles"."require_mfa" = TRUE"#), "{sql}");
        assert!(
            sql.contains(&format!(r#""roles"."client_id" = '{client}'"#)),
            "{sql}"
        );
    }

    #[test]
    fn ids_filter_is_an_in_list() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let sql = sql(&RoleFilter {
            ids: Some(vec![first, second]),
            ..RoleFilter::default()
        });
        assert!(
            sql.contains(&format!(r#""roles"."id" IN ('{first}', '{second}')"#)),
            "{sql}"
        );
    }
}
