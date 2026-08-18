use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    prelude::Expr,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::authentication::entities::AuthenticationError;
use crate::domain::authentication::ports::{LoginActionToken, LoginActionTokenRepository};
use crate::entity::login_action_tokens::{
    ActiveModel as LatActiveModel, Column as LatColumn, Entity as LatEntity, Model as LatModel,
};

impl From<LatModel> for LoginActionToken {
    fn from(model: LatModel) -> Self {
        LoginActionToken {
            jti: model.jti,
            user_id: model.user_id,
            realm_id: model.realm_id,
            auth_session_id: model.auth_session_id,
            expires_at: model.expires_at.and_utc(),
            consumed_at: model.consumed_at.map(|value| value.and_utc()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PostgresLoginActionTokenRepository {
    pub db: DatabaseConnection,
}

impl PostgresLoginActionTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl LoginActionTokenRepository for PostgresLoginActionTokenRepository {
    async fn create(&self, token: LoginActionToken) -> Result<(), AuthenticationError> {
        let model = LatActiveModel {
            jti: Set(token.jti),
            user_id: Set(token.user_id),
            realm_id: Set(token.realm_id),
            auth_session_id: Set(token.auth_session_id),
            expires_at: Set(token.expires_at.naive_utc()),
            consumed_at: Set(None),
            created_at: Set(Utc::now().naive_utc()),
        };

        model.insert(&self.db).await.map_err(|e| {
            error!("Error creating login action token: {e:?}");
            AuthenticationError::InternalServerError
        })?;

        Ok(())
    }

    async fn get_by_jti(&self, jti: Uuid) -> Result<Option<LoginActionToken>, AuthenticationError> {
        let model = LatEntity::find_by_id(jti)
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error loading login action token: {e:?}");
                AuthenticationError::InternalServerError
            })?;

        Ok(model.map(Into::into))
    }

    async fn consume(&self, jti: Uuid) -> Result<bool, AuthenticationError> {
        let now: DateTime<Utc> = Utc::now();

        let claimed = LatEntity::update_many()
            .col_expr(LatColumn::ConsumedAt, Expr::value(now.naive_utc()))
            .filter(LatColumn::Jti.eq(jti))
            .filter(LatColumn::ConsumedAt.is_null())
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error consuming login action token: {e:?}");
                AuthenticationError::InternalServerError
            })?;

        Ok(claimed.rows_affected > 0)
    }
}
