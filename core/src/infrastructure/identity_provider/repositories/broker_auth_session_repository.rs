use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tracing::instrument;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::domain::identity_provider::broker::{
    BrokerAuthSession, BrokerAuthSessionRepository, CreateBrokerAuthSessionRequest,
};
use crate::entity::broker_auth_sessions::{ActiveModel, Column, Entity as BrokerAuthSessionEntity};

/// PostgreSQL implementation of the BrokerAuthSessionRepository trait
#[derive(Debug, Clone)]
pub struct PostgresBrokerAuthSessionRepository {
    db: DatabaseConnection,
}

impl PostgresBrokerAuthSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl BrokerAuthSessionRepository for PostgresBrokerAuthSessionRepository {
    #[instrument(skip(self, request), fields(realm_id = ?request.realm_id, broker_state = %request.broker_state))]
    async fn create(
        &self,
        request: CreateBrokerAuthSessionRequest,
    ) -> Result<BrokerAuthSession, CoreError> {
        let now = Utc::now().fixed_offset();
        let expires_at = (Utc::now() + chrono::Duration::minutes(10)).fixed_offset();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(request.realm_id),
            identity_provider_id: Set(request.identity_provider_id),
            client_id: Set(request.client_id),
            redirect_uri: Set(request.redirect_uri),
            response_type: Set(request.response_type),
            scope: Set(request.scope),
            state: Set(request.state),
            nonce: Set(request.nonce),
            broker_state: Set(request.broker_state),
            code_verifier: Set(request.code_verifier),
            auth_session_id: Set(request.auth_session_id),
            created_at: Set(now),
            expires_at: Set(expires_at),
        };

        let session = payload.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to create broker auth session: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(session.into())
    }

    #[instrument(skip(self), fields(broker_state = %broker_state))]
    async fn get_by_broker_state(
        &self,
        broker_state: &str,
    ) -> Result<Option<BrokerAuthSession>, CoreError> {
        let session = BrokerAuthSessionEntity::find()
            .filter(Column::BrokerState.eq(broker_state))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get broker auth session by state: {}", e);
                CoreError::InternalServerError
            })?
            .map(BrokerAuthSession::from);

        Ok(session)
    }

    #[instrument(skip(self), fields(session_id = %id))]
    async fn get_by_id(&self, id: Uuid) -> Result<Option<BrokerAuthSession>, CoreError> {
        let session = BrokerAuthSessionEntity::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get broker auth session by id: {}", e);
                CoreError::InternalServerError
            })?
            .map(BrokerAuthSession::from);

        Ok(session)
    }

    #[instrument(skip(self), fields(session_id = %id))]
    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        BrokerAuthSessionEntity::delete_many()
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete broker auth session: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_expired(&self) -> Result<u64, CoreError> {
        let now = Utc::now().fixed_offset();

        let result = BrokerAuthSessionEntity::delete_many()
            .filter(Column::ExpiresAt.lt(now))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete expired broker auth sessions: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }
}

/// Convert from SeaORM model to domain entity
impl From<crate::entity::broker_auth_sessions::Model> for BrokerAuthSession {
    fn from(model: crate::entity::broker_auth_sessions::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id.into(),
            identity_provider_id: model.identity_provider_id.into(),
            client_id: model.client_id,
            redirect_uri: model.redirect_uri,
            response_type: model.response_type,
            scope: model.scope,
            state: model.state,
            nonce: model.nonce,
            broker_state: model.broker_state,
            code_verifier: model.code_verifier,
            auth_session_id: model.auth_session_id,
            created_at: model.created_at.with_timezone(&Utc),
            expires_at: model.expires_at.with_timezone(&Utc),
        }
    }
}
