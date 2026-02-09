use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::aegis::entities::{ClientScope, ClientScopeMapping};
use crate::domain::aegis::ports::ClientScopeMappingRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::{client_scope_mappings, client_scopes};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostgresScopeMappingRepository {
    pub db: DatabaseConnection,
}

impl PostgresScopeMappingRepository {
    #[allow(dead_code)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ClientScopeMappingRepository for PostgresScopeMappingRepository {
    async fn assign_scope_to_client(
        &self,
        client_id: Uuid,
        scope_id: Uuid,
        is_default: bool,
        is_optional: bool,
    ) -> Result<ClientScopeMapping, CoreError> {
        let active_model = client_scope_mappings::ActiveModel {
            client_id: Set(client_id),
            scope_id: Set(scope_id),
            is_default: Set(is_default),
            is_optional: Set(is_optional),
        };

        let model = active_model.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to assign scope to client: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    async fn remove_scope_from_client(
        &self,
        client_id: Uuid,
        scope_id: Uuid,
    ) -> Result<(), CoreError> {
        let result = client_scope_mappings::Entity::delete_many()
            .filter(client_scope_mappings::Column::ClientId.eq(client_id))
            .filter(client_scope_mappings::Column::ScopeId.eq(scope_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to remove scope from client: {}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn get_client_scopes(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<ClientScopeMapping>, CoreError> {
        let models = client_scope_mappings::Entity::find()
            .filter(client_scope_mappings::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get client scope mappings: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(ClientScopeMapping::from).collect())
    }

    async fn get_default_scopes(&self, client_id: Uuid) -> Result<Vec<ClientScope>, CoreError> {
        let mappings = client_scope_mappings::Entity::find()
            .filter(client_scope_mappings::Column::ClientId.eq(client_id))
            .filter(client_scope_mappings::Column::IsDefault.eq(true))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get default scope mappings: {}", e);
                CoreError::InternalServerError
            })?;

        let scope_ids: Vec<Uuid> = mappings.iter().map(|m| m.scope_id).collect();

        if scope_ids.is_empty() {
            return Ok(vec![]);
        }

        let scopes = client_scopes::Entity::find()
            .filter(client_scopes::Column::Id.is_in(scope_ids))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get default scopes: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(scopes.into_iter().map(ClientScope::from).collect())
    }

    async fn get_optional_scopes(&self, client_id: Uuid) -> Result<Vec<ClientScope>, CoreError> {
        let mappings = client_scope_mappings::Entity::find()
            .filter(client_scope_mappings::Column::ClientId.eq(client_id))
            .filter(client_scope_mappings::Column::IsOptional.eq(true))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get optional scope mappings: {}", e);
                CoreError::InternalServerError
            })?;

        let scope_ids: Vec<Uuid> = mappings.iter().map(|m| m.scope_id).collect();

        if scope_ids.is_empty() {
            return Ok(vec![]);
        }

        let scopes = client_scopes::Entity::find()
            .filter(client_scopes::Column::Id.is_in(scope_ids))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get optional scopes: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(scopes.into_iter().map(ClientScope::from).collect())
    }
}
