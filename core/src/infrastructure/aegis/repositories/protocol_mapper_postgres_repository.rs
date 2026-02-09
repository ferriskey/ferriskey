use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::aegis::entities::ProtocolMapper;
use crate::domain::aegis::ports::ProtocolMapperRepository;
use crate::domain::aegis::value_objects::{
    CreateProtocolMapperRequest, UpdateProtocolMapperRequest,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::entity::client_scope_protocol_mappers;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostgresProtocolMapperRepository {
    pub db: DatabaseConnection,
}

impl PostgresProtocolMapperRepository {
    #[allow(dead_code)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ProtocolMapperRepository for PostgresProtocolMapperRepository {
    async fn create(
        &self,
        payload: CreateProtocolMapperRequest,
    ) -> Result<ProtocolMapper, CoreError> {
        let now = Utc::now().naive_utc();

        let active_model = client_scope_protocol_mappers::ActiveModel {
            id: Set(generate_uuid_v7()),
            client_scope_id: Set(payload.client_scope_id),
            name: Set(payload.name),
            mapper_type: Set(payload.mapper_type),
            config: Set(payload.config),
            created_at: Set(now),
        };

        let model = active_model.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to insert protocol mapper: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<ProtocolMapper>, CoreError> {
        let model = client_scope_protocol_mappers::Entity::find()
            .filter(client_scope_protocol_mappers::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get protocol mapper by id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(ProtocolMapper::from))
    }

    async fn get_by_scope_id(&self, scope_id: Uuid) -> Result<Vec<ProtocolMapper>, CoreError> {
        let models = client_scope_protocol_mappers::Entity::find()
            .filter(client_scope_protocol_mappers::Column::ClientScopeId.eq(scope_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get protocol mappers by scope id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(ProtocolMapper::from).collect())
    }

    async fn update_by_id(
        &self,
        id: Uuid,
        payload: UpdateProtocolMapperRequest,
    ) -> Result<ProtocolMapper, CoreError> {
        let model = client_scope_protocol_mappers::Entity::find()
            .filter(client_scope_protocol_mappers::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find protocol mapper for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active: client_scope_protocol_mappers::ActiveModel = model.into();

        active.name = match payload.name {
            Some(v) => Set(v),
            None => active.name,
        };
        active.mapper_type = match payload.mapper_type {
            Some(v) => Set(v),
            None => active.mapper_type,
        };
        active.config = match payload.config {
            Some(v) => Set(v),
            None => active.config,
        };

        let model = active.update(&self.db).await.map_err(|e| {
            tracing::error!("Failed to update protocol mapper: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), CoreError> {
        let result = client_scope_protocol_mappers::Entity::delete_many()
            .filter(client_scope_protocol_mappers::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete protocol mapper: {}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
