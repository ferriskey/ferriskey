use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::OnConflict,
};
use uuid::Uuid;

use ferriskey_consent::{ConsentDecision, ConsentDecisionId, ConsentDecisionRepository};
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;

use crate::entity::{clients, consent_session_scopes, consent_sessions, realm_settings};

#[derive(Debug, Clone)]
pub struct PostgresConsentDecisionRepository {
    pub db: DatabaseConnection,
}

impl PostgresConsentDecisionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn into_domain(
    model: consent_sessions::Model,
    scopes: Vec<consent_session_scopes::Model>,
) -> ConsentDecision {
    let (granted, denied): (Vec<_>, Vec<_>) = scopes.into_iter().partition(|scope| scope.granted);

    ConsentDecision {
        id: ConsentDecisionId(model.id),
        realm_id: model.realm_id.into(),
        user_id: model.user_id,
        client_id: model.client_id,
        granted_scopes: granted.into_iter().map(|scope| scope.scope).collect(),
        denied_scopes: denied.into_iter().map(|scope| scope.scope).collect(),
        expires_at: Utc.from_utc_datetime(&model.expires_at),
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

fn scope_rows(
    consent_session_id: Uuid,
    decision: &ConsentDecision,
) -> Vec<consent_session_scopes::ActiveModel> {
    decision
        .granted_scopes
        .iter()
        .map(|scope| (scope, true))
        .chain(decision.denied_scopes.iter().map(|scope| (scope, false)))
        .map(|(scope, granted)| consent_session_scopes::ActiveModel {
            consent_session_id: Set(consent_session_id),
            scope: Set(scope.clone()),
            granted: Set(granted),
        })
        .collect()
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

        let Some(model) = model else {
            return Ok(None);
        };

        let scopes = consent_session_scopes::Entity::find()
            .filter(consent_session_scopes::Column::ConsentSessionId.eq(model.id))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(Some(into_domain(model, scopes)))
    }

    async fn upsert(&self, decision: ConsentDecision) -> Result<ConsentDecision, CoreError> {
        let payload = consent_sessions::ActiveModel {
            id: Set(decision.id.0),
            realm_id: Set(decision.realm_id.into()),
            user_id: Set(decision.user_id),
            client_id: Set(decision.client_id),
            expires_at: Set(decision.expires_at.naive_utc()),
            created_at: Set(decision.created_at.naive_utc()),
            updated_at: Set(decision.updated_at.naive_utc()),
        };

        let transaction = self
            .db
            .begin()
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let model = consent_sessions::Entity::insert(payload)
            .on_conflict(
                OnConflict::columns([
                    consent_sessions::Column::RealmId,
                    consent_sessions::Column::UserId,
                    consent_sessions::Column::ClientId,
                ])
                .update_columns([
                    consent_sessions::Column::ExpiresAt,
                    consent_sessions::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec_with_returning(&transaction)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        consent_session_scopes::Entity::delete_many()
            .filter(consent_session_scopes::Column::ConsentSessionId.eq(model.id))
            .exec(&transaction)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let rows = scope_rows(model.id, &decision);
        if !rows.is_empty() {
            consent_session_scopes::Entity::insert_many(rows)
                .exec(&transaction)
                .await
                .map_err(|_| CoreError::InternalServerError)?;
        }

        let scopes = consent_session_scopes::Entity::find()
            .filter(consent_session_scopes::Column::ConsentSessionId.eq(model.id))
            .all(&transaction)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        transaction
            .commit()
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(into_domain(model, scopes))
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
