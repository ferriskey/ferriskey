use chrono::{DateTime, TimeZone, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::{
    common::generate_uuid_v7,
    jwt::{JwtError, entities::AccessToken, ports::AccessTokenRepository},
};

impl From<crate::entity::access_tokens::Model> for AccessToken {
    fn from(model: crate::entity::access_tokens::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = model.expires_at.map(|dt| Utc.from_utc_datetime(&dt));

        AccessToken::new(
            model.id,
            model.token_hash,
            model.jti,
            model.user_id,
            model.realm_id,
            model.revoked,
            expires_at,
            model.claims,
            created_at,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PostgresAccessTokenRepository {
    pub db: DatabaseConnection,
}

impl PostgresAccessTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl AccessTokenRepository for PostgresAccessTokenRepository {
    async fn create(
        &self,
        token_hash: String,
        jti: Option<Uuid>,
        user_id: Uuid,
        realm_id: ferriskey_domain::realm::RealmId,
        expires_at: Option<DateTime<Utc>>,
        claims: serde_json::Value,
    ) -> Result<AccessToken, JwtError> {
        let model = crate::entity::access_tokens::ActiveModel {
            id: Set(generate_uuid_v7()),
            token_hash: Set(token_hash),
            jti: Set(jti),
            user_id: Set(user_id),
            realm_id: Set(realm_id.into()),
            revoked: Set(false),
            expires_at: Set(expires_at.map(|dt| dt.naive_utc())),
            claims: Set(claims),
            created_at: Set(Utc::now().naive_utc()),
        };

        let access_token = model
            .insert(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(access_token.into())
    }

    async fn get_by_token_hash(&self, token_hash: String) -> Result<Option<AccessToken>, JwtError> {
        let access_token = crate::entity::access_tokens::Entity::find()
            .filter(crate::entity::access_tokens::Column::TokenHash.eq(token_hash))
            .one(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(access_token.map(|t| t.into()))
    }

    async fn revoke_by_token_hash(&self, token_hash: String) -> Result<(), JwtError> {
        crate::entity::access_tokens::Entity::update_many()
            .col_expr(
                crate::entity::access_tokens::Column::Revoked,
                Expr::value(true),
            )
            .filter(crate::entity::access_tokens::Column::TokenHash.eq(token_hash))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }
}
