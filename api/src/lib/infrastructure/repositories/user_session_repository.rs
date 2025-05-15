use chrono::{TimeZone, Utc};
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, QueryFilter};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::session::{
    entities::{error::SessionError, model::UserSession},
    ports::user_session_repository::UserSessionRepository,
};

impl From<entity::user_sessions::Model> for UserSession {
    fn from(model: entity::user_sessions::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = Utc.from_utc_datetime(&model.expires_at);

        UserSession {
            id: model.id,
            user_id: model.user_id,
            realm_id: model.realm_id,
            user_agent: model.user_agent,
            ip_address: model.ip_address,
            created_at,
            expires_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresUserSessionRepository {
    pool: PgPool,
    pub db: DatabaseConnection,
}

impl PostgresUserSessionRepository {
    pub fn new(pool: PgPool, db: DatabaseConnection) -> Self {
        Self { pool, db }
    }
}

impl UserSessionRepository for PostgresUserSessionRepository {
    async fn create(&self, session: &UserSession) -> Result<(), SessionError> {
        let active_model = entity::user_sessions::ActiveModel {
            id: Set(session.id),
            user_id: Set(session.user_id),
            realm_id: Set(session.realm_id),
            user_agent: Set(session.user_agent.clone()),
            ip_address: Set(session.ip_address.clone()),
            created_at: Set(session.created_at.naive_utc()),
            expires_at: Set(session.expires_at.naive_utc()),
        };

        active_model
            .insert(&self.db)
            .await
            .map_err(|_| SessionError::CreateError)?;

        Ok(())
    }

    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<UserSession, SessionError> {
        let user_session = entity::user_sessions::Entity::find()
            .filter(entity::user_sessions::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|_| SessionError::NotFound)?
            .ok_or(SessionError::NotFound)?;

        Ok(user_session.into())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), SessionError> {
        entity::user_sessions::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|_| SessionError::DeleteError)?;

        Ok(())
    }
}
