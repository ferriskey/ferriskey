use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use uuid::Uuid;

use crate::domain::authentication::backchannel_logout::{
    SessionParticipant, SessionParticipantRepository,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

// The access_tokens branch covers tokens issued by older replicas during a
// rolling upgrade, after the migration backfilled session_participants.
const PARTICIPANTS_SQL: &str = "\
    SELECT client_id, issuer FROM session_participants \
    WHERE realm_id = $1 AND session_id = $2 \
    UNION \
    SELECT claims ->> 'azp', claims ->> 'iss' FROM access_tokens \
    WHERE realm_id = $1 AND claims ->> 'sid' = $2::text \
      AND claims ->> 'azp' IS NOT NULL AND claims ->> 'iss' IS NOT NULL";

/// Reads the participants recorded for a session when its tokens were issued:
/// the client (`azp`) and the realm URL (`iss`) each token was issued under.
/// The rows go with the session, so this is read before the session is deleted.
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
                PARTICIPANTS_SQL,
                [realm.into(), session_id.into()],
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
