use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use uuid::Uuid;

use crate::domain::authentication::backchannel_logout::{
    SessionParticipant, SessionParticipantRepository,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

/// Reads the participants of a session off the access tokens issued in it:
/// each one stores its claims, so `azp` names the client and `iss` the realm
/// URL it was issued under. Revoked tokens count too, since a session's
/// tokens are revoked before the session row goes.
#[derive(Debug, Clone)]
pub struct PostgresSessionParticipantRepository {
    db: DatabaseConnection,
}

impl PostgresSessionParticipantRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SessionParticipantRepository for PostgresSessionParticipantRepository {
    async fn participants(
        &self,
        realm_id: RealmId,
        session_id: Uuid,
    ) -> Result<Vec<SessionParticipant>, CoreError> {
        let realm: Uuid = realm_id.into();
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT DISTINCT claims ->> 'azp' AS client_id, claims ->> 'iss' AS issuer \
                 FROM access_tokens \
                 WHERE realm_id = $1 AND claims ->> 'sid' = $2 \
                   AND claims ->> 'azp' IS NOT NULL AND claims ->> 'iss' IS NOT NULL",
                [realm.into(), session_id.to_string().into()],
            ))
            .await
            .map_err(|err| {
                tracing::error!(error = %err, "Failed to list the clients of a session");
                CoreError::InternalServerError
            })?;

        Ok(rows
            .iter()
            .filter_map(|row| {
                Some(SessionParticipant {
                    client_id: row.try_get::<String>("", "client_id").ok()?,
                    issuer: row.try_get::<String>("", "issuer").ok()?,
                })
            })
            .collect())
    }
}
