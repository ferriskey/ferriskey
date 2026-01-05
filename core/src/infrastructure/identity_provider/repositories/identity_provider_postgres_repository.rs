use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use tracing::instrument;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::domain::identity_provider::{
    CreateIdentityProviderRequest, IdentityProvider, IdentityProviderRepository,
    UpdateIdentityProviderRequest,
};
use crate::domain::realm::entities::RealmId;
use crate::entity::identity_providers::{ActiveModel, Column, Entity as IdentityProviderEntity};

#[derive(Debug, Clone)]
pub struct PostgresIdentityProviderRepository {
    pub db: DatabaseConnection,
}

impl PostgresIdentityProviderRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl IdentityProviderRepository for PostgresIdentityProviderRepository {
    #[instrument(skip(self))]
    async fn create(
        &self,
        request: CreateIdentityProviderRequest,
    ) -> Result<IdentityProvider, CoreError> {
        let now = chrono::Utc::now().fixed_offset();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(request.realm_id.into()),
            alias: Set(request.alias),
            provider_id: Set(request.provider_id),
            enabled: Set(request.enabled),
            display_name: Set(request.display_name),
            first_broker_login_flow_alias: Set(request.first_broker_login_flow_alias),
            post_broker_login_flow_alias: Set(request.post_broker_login_flow_alias),
            store_token: Set(request.store_token),
            add_read_token_role_on_create: Set(request.add_read_token_role_on_create),
            trust_email: Set(request.trust_email),
            link_only: Set(request.link_only),
            config: Set(request.config),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let identity_provider = payload.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to insert identity provider: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(identity_provider.into())
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<IdentityProvider>, CoreError> {
        let identity_provider = IdentityProviderEntity::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find identity provider by id: {}", e);
                CoreError::InternalServerError
            })?
            .map(IdentityProvider::from);

        Ok(identity_provider)
    }

    #[instrument(skip(self))]
    async fn find_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<Option<IdentityProvider>, CoreError> {
        let identity_provider = IdentityProviderEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(Column::Alias.eq(alias))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find identity provider by realm and alias: {}", e);
                CoreError::InternalServerError
            })?
            .map(IdentityProvider::from);

        Ok(identity_provider)
    }

    #[instrument(skip(self))]
    async fn find_by_realm(&self, realm_id: RealmId) -> Result<Vec<IdentityProvider>, CoreError> {
        let identity_providers = IdentityProviderEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .order_by_asc(Column::Alias)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find identity providers by realm: {}", e);
                CoreError::InternalServerError
            })?;

        let identity_providers: Vec<IdentityProvider> = identity_providers
            .into_iter()
            .map(IdentityProvider::from)
            .collect();

        Ok(identity_providers)
    }

    #[instrument(skip(self))]
    async fn update(
        &self,
        id: Uuid,
        request: UpdateIdentityProviderRequest,
    ) -> Result<IdentityProvider, CoreError> {
        let existing = IdentityProviderEntity::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find identity provider for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::ProviderNotFound)?;

        let mut identity_provider: ActiveModel = existing.into();

        if let Some(enabled) = request.enabled {
            identity_provider.enabled = Set(enabled);
        }
        if let Some(display_name) = request.display_name {
            identity_provider.display_name = Set(Some(display_name));
        }
        if let Some(first_broker_login_flow_alias) = request.first_broker_login_flow_alias {
            identity_provider.first_broker_login_flow_alias =
                Set(Some(first_broker_login_flow_alias));
        }
        if let Some(post_broker_login_flow_alias) = request.post_broker_login_flow_alias {
            identity_provider.post_broker_login_flow_alias =
                Set(Some(post_broker_login_flow_alias));
        }
        if let Some(store_token) = request.store_token {
            identity_provider.store_token = Set(store_token);
        }
        if let Some(add_read_token_role_on_create) = request.add_read_token_role_on_create {
            identity_provider.add_read_token_role_on_create = Set(add_read_token_role_on_create);
        }
        if let Some(trust_email) = request.trust_email {
            identity_provider.trust_email = Set(trust_email);
        }
        if let Some(link_only) = request.link_only {
            identity_provider.link_only = Set(link_only);
        }
        if let Some(config) = request.config {
            identity_provider.config = Set(config);
        }

        identity_provider.updated_at = Set(chrono::Utc::now().fixed_offset());

        let updated = identity_provider.update(&self.db).await.map_err(|e| {
            tracing::error!("Failed to update identity provider: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(updated.into())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        let result = IdentityProviderEntity::delete_many()
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete identity provider: {}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::ProviderNotFound);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn exists_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<bool, CoreError> {
        let count = IdentityProviderEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(Column::Alias.eq(alias))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check identity provider existence: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(count > 0)
    }
}
