use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use uuid::Uuid;

use ferriskey_consent::{ConsentDecision, ConsentDecisionId, ConsentDecisionRepository};
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;

use crate::entity::{clients, consent_sessions, realm_settings};

#[derive(Debug, Clone)]
pub struct PostgresConsentDecisionRepository {
    pub db: DatabaseConnection,
}

impl PostgresConsentDecisionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn into_domain(model: consent_sessions::Model) -> ConsentDecision {
    ConsentDecision {
        id: ConsentDecisionId(model.id),
        realm_id: model.realm_id.into(),
        user_id: model.user_id,
        client_id: model.client_id,
        granted_scopes: model.granted_scopes,
        denied_scopes: model.denied_scopes,
        expires_at: Utc.from_utc_datetime(&model.expires_at),
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

impl ConsentDecisionRepository for PostgresConsentDecisionRepository {
    async fn find(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<ConsentDecision>, CoreError> {
        let realm_id: Uuid = realm_id.into();

        let model = consent_sessions::Entity::find()
            .filter(consent_sessions::Column::RealmId.eq(realm_id))
            .filter(consent_sessions::Column::UserId.eq(user_id))
            .filter(consent_sessions::Column::ClientId.eq(client_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(model.map(into_domain))
    }

    async fn upsert(&self, decision: ConsentDecision) -> Result<ConsentDecision, CoreError> {
        let payload = consent_sessions::ActiveModel {
            id: Set(decision.id.0),
            realm_id: Set(decision.realm_id.into()),
            user_id: Set(decision.user_id),
            client_id: Set(decision.client_id),
            granted_scopes: Set(decision.granted_scopes),
            denied_scopes: Set(decision.denied_scopes),
            expires_at: Set(decision.expires_at.naive_utc()),
            created_at: Set(decision.created_at.naive_utc()),
            updated_at: Set(decision.updated_at.naive_utc()),
        };

        let model = consent_sessions::Entity::insert(payload)
            .on_conflict(
                OnConflict::columns([
                    consent_sessions::Column::RealmId,
                    consent_sessions::Column::UserId,
                    consent_sessions::Column::ClientId,
                ])
                .update_columns([
                    consent_sessions::Column::GrantedScopes,
                    consent_sessions::Column::DeniedScopes,
                    consent_sessions::Column::ExpiresAt,
                    consent_sessions::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(into_domain(model))
    }

    async fn get_client_consent_required(&self, client_id: Uuid) -> Result<bool, CoreError> {
        let model = clients::Entity::find_by_id(client_id)
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::ClientNotFound)?;

        Ok(model.consent_required)
    }

    async fn get_consent_ttl_days(&self, realm_id: RealmId) -> Result<Option<i32>, CoreError> {
        let realm_id: Uuid = realm_id.into();

        let model = realm_settings::Entity::find()
            .filter(realm_settings::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(model.and_then(|model| model.consent_ttl_days))
    }
}
