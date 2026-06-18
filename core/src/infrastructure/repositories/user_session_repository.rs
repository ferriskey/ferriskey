use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    prelude::Expr,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::session::{
    entities::{SessionError, UserSession},
    ports::UserSessionRepository,
};

impl From<crate::entity::user_sessions::Model> for UserSession {
    fn from(model: crate::entity::user_sessions::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = Utc.from_utc_datetime(&model.expires_at);
        let last_seen_at = model.last_seen_at.map(|ref dt| Utc.from_utc_datetime(dt));

        UserSession {
            id: model.id,
            user_id: model.user_id,
            realm_id: model.realm_id,
            user_agent: model.user_agent,
            ip_address: model.ip_address,
            created_at,
            expires_at,
            last_seen_at,
            soft_expiry_duration: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PostgresUserSessionRepository {
    pub db: DatabaseConnection,
}

impl PostgresUserSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserSessionRepository for PostgresUserSessionRepository {
    async fn create(&self, session: &UserSession) -> Result<(), SessionError> {
        let model = crate::entity::user_sessions::ActiveModel {
            id: Set(session.id),
            user_id: Set(session.user_id),
            realm_id: Set(session.realm_id),
            user_agent: Set(session.user_agent.clone()),
            ip_address: Set(session.ip_address.clone()),
            created_at: Set(session.created_at.naive_utc()),
            expires_at: Set(session.expires_at.naive_utc()),
            last_seen_at: Set(session.last_seen_at.map(|dt| dt.naive_utc())),
        };

        model.insert(&self.db).await.map_err(|e| {
            error!("Error creating user session: {:?}", e);
            SessionError::CreateError
        })?;

        Ok(())
    }

    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<UserSession, SessionError> {
        let session = crate::entity::user_sessions::Entity::find()
            .filter(crate::entity::user_sessions::Column::UserId.eq(*user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding user session: {:?}", e);
                SessionError::NotFound
            })?
            .ok_or(SessionError::NotFound)?;

        Ok(session.into())
    }

    async fn find_all_by_user_and_realm(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Vec<UserSession>, SessionError> {
        let sessions = crate::entity::user_sessions::Entity::find()
            .filter(crate::entity::user_sessions::Column::UserId.eq(user_id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Error listing user sessions: {:?}", e);
                SessionError::NotFound
            })?;

        Ok(sessions.into_iter().map(|m| m.into()).collect())
    }

    async fn find_by_id(&self, session_id: Uuid) -> Result<Option<UserSession>, SessionError> {
        let session = crate::entity::user_sessions::Entity::find()
            .filter(crate::entity::user_sessions::Column::Id.eq(session_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding user session by id: {:?}", e);
                SessionError::NotFound
            })?;

        Ok(session.map(|m| m.into()))
    }

    async fn delete(&self, id: &Uuid) -> Result<(), SessionError> {
        crate::entity::user_sessions::Entity::delete_many()
            .filter(crate::entity::user_sessions::Column::Id.eq(*id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error deleting user session: {:?}", e);
                SessionError::DeleteError
            })?;

        Ok(())
    }

    async fn update_last_seen(&self, session_id: Uuid) -> Result<(), SessionError> {
        crate::entity::user_sessions::Entity::update_many()
            .col_expr(
                crate::entity::user_sessions::Column::LastSeenAt,
                Expr::value(Utc::now().naive_utc()),
            )
            .filter(crate::entity::user_sessions::Column::Id.eq(session_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating last_seen_at for user session: {:?}", e);
                SessionError::DeleteError
            })?;

        Ok(())
    }
}
