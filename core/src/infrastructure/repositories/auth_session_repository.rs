use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    prelude::Expr,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::authentication::{
    entities::{AuthSession, AuthenticationError},
    ports::AuthSessionRepository,
};

impl From<crate::entity::auth_sessions::Model> for AuthSession {
    fn from(model: crate::entity::auth_sessions::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = Utc.from_utc_datetime(&model.expires_at);

        AuthSession {
            id: model.id,
            realm_id: model.realm_id,
            client_id: model.client_id,
            redirect_uri: model.redirect_uri,
            response_type: model.response_type,
            scope: model.scope,
            state: model.state,
            nonce: model.nonce,
            code: model.code,
            authenticated: model.authenticated,
            user_id: model.user_id,
            created_at,
            expires_at,
        }
    }
}

#[derive(Clone)]
pub struct PostgresAuthSessionRepository {
    pub db: DatabaseConnection,
}

impl PostgresAuthSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl AuthSessionRepository for PostgresAuthSessionRepository {
    async fn create(&self, session: &AuthSession) -> Result<AuthSession, AuthenticationError> {
        let model = crate::entity::auth_sessions::ActiveModel {
            id: Set(session.id),
            realm_id: Set(session.realm_id),
            client_id: Set(session.client_id),
            redirect_uri: Set(session.redirect_uri.clone()),
            response_type: Set(session.response_type.clone()),
            scope: Set(session.scope.clone()),
            state: Set(session.state.clone()),
            nonce: Set(session.nonce.clone()),
            code: Set(session.code.clone()),
            authenticated: Set(false),
            user_id: Set(None),
            created_at: Set(session.created_at.naive_utc()),
            expires_at: Set(session.expires_at.naive_utc()),
        };

        let t = model
            .insert(&self.db)
            .await
            .map_err(|e| {
                error!("Error creating session: {:?}", e);
                AuthenticationError::InternalServerError
            })?
            .into();

        Ok(t)
    }

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::find()
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error getting session: {:?}", e);
                AuthenticationError::NotFound
            })?;

        let session = session.ok_or(AuthenticationError::NotFound)?.into();

        Ok(session)
    }

    async fn get_by_code(&self, code: String) -> Result<Option<AuthSession>, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::find()
            .filter(crate::entity::auth_sessions::Column::Code.eq(code))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error getting session: {:?}", e);
                AuthenticationError::NotFound
            })?;

        let session: Option<AuthSession> = session.map(|s| s.into());

        Ok(session)
    }

    async fn update_code_and_user_id(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::update_many()
            .col_expr(
                crate::entity::auth_sessions::Column::Code,
                Expr::value(code),
            )
            .col_expr(
                crate::entity::auth_sessions::Column::UserId,
                Expr::value(user_id),
            )
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating session: {:?}", e);
                AuthenticationError::Invalid
            })?
            .into_iter()
            .next()
            .ok_or(AuthenticationError::NotFound)?
            .into();

        Ok(session)
    }
}
