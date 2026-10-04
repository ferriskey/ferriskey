use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::scope::RealmScope;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    Select, TransactionTrait,
};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::{RealmId, Unscoped};
use crate::domain::seawatch::{
    entities::SecurityEvent,
    hashing::{GENESIS_PREV_HASH, compute_event_hash},
    pii::{AuditPiiMode, PiiConfig, apply_to_event},
    ports::SecurityEventRepository,
    value_objects::{SecurityEventFilter, SecurityEventSortField},
};
use crate::entity::security_events;
use crate::infrastructure::pagination::{SortColumn, contains, paginate};

#[derive(Debug, Clone)]
pub struct PostgresSecurityEventRepository {
    pub db: DatabaseConnection,
}

impl PostgresSecurityEventRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn load_pii_config(&self, realm_id: Uuid) -> PiiConfig {
        let result = crate::entity::realm_settings::Entity::find()
            .filter(crate::entity::realm_settings::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await;

        match result {
            Ok(Some(model)) => {
                let mode = model
                    .seawatch_pii_mode
                    .parse::<AuditPiiMode>()
                    .unwrap_or_default();
                PiiConfig {
                    mode,
                    pseudo_key: model.seawatch_pseudo_key,
                }
            }
            _ => PiiConfig::default(),
        }
    }
}

impl SortColumn<security_events::Entity> for SecurityEventSortField {
    fn column(&self) -> security_events::Column {
        match self {
            SecurityEventSortField::EventType => security_events::Column::EventType,
            SecurityEventSortField::Status => security_events::Column::Status,
            SecurityEventSortField::Timestamp => security_events::Column::Timestamp,
            SecurityEventSortField::CreatedAt => security_events::Column::CreatedAt,
        }
    }
}

fn listing_select(realm_id: Uuid, filter: &SecurityEventFilter) -> Select<security_events::Entity> {
    use security_events::Column;

    security_events::Entity::find()
        .filter(Column::RealmId.eq(realm_id))
        .apply_if(filter.actor_id, |select, value| {
            select.filter(Column::ActorId.eq(value))
        })
        .apply_if(filter.client_id, |select, value| {
            select.filter(Column::TargetId.eq(value))
        })
        .apply_if(filter.event_types.as_deref(), |select, values| {
            select.filter(Column::EventType.is_in(values.iter().map(ToString::to_string)))
        })
        .apply_if(filter.status.as_ref(), |select, value| {
            select.filter(Column::Status.eq(value.to_string()))
        })
        .apply_if(filter.target_type.as_deref(), |select, value| {
            select.filter(Column::TargetType.eq(value))
        })
        .apply_if(filter.ip_address.as_deref(), |select, value| {
            select.filter(contains(Column::IpAddress, value))
        })
        .apply_if(filter.from_timestamp, |select, value| {
            select.filter(Column::Timestamp.gte(value.naive_utc()))
        })
        .apply_if(filter.to_timestamp, |select, value| {
            select.filter(Column::Timestamp.lte(value.naive_utc()))
        })
}

impl SecurityEventRepository for PostgresSecurityEventRepository {
    /// Opens a transaction, fetches the current chain head (latest event for
    /// the realm ordered by `created_at DESC`) under an exclusive lock,
    /// computes `event_hash = SHA-256(preimage || prev_hash)`, and inserts in
    /// the same transaction to keep the chain linear even under concurrent
    /// writes from multiple API workers (Postgres serialises the writes
    /// within the txn).
    async fn store_event(&self, mut event: SecurityEvent) -> Result<(), CoreError> {
        let realm_id: Uuid = event.realm_id.into();
        let pii_cfg = self.load_pii_config(realm_id).await;
        event = apply_to_event(event, &pii_cfg);

        let txn = self.db.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction for chained event: {}", e);
            CoreError::InternalServerError
        })?;

        // Lock the latest row for this realm so concurrent inserts are serialised.
        let head = security_events::Entity::find()
            .filter(security_events::Column::RealmId.eq(realm_id))
            .order_by_desc(security_events::Column::CreatedAt)
            .lock_exclusive()
            .one(&txn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch chain head: {}", e);
                CoreError::InternalServerError
            })?;

        let prev_hash: [u8; 32] = head
            .and_then(|m| {
                m.event_hash
                    .and_then(|h| hex::decode(&h).ok().and_then(|b| b.try_into().ok()))
            })
            .unwrap_or(GENESIS_PREV_HASH);

        let hash = compute_event_hash(&event, &prev_hash);
        event.prev_hash = Some(prev_hash);
        event.event_hash = Some(hash);

        let active_model: security_events::ActiveModel = event.into();
        security_events::Entity::insert(active_model)
            .exec(&txn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to store chained security event: {}", e);
                CoreError::InternalServerError
            })?;

        txn.commit().await.map_err(|e| {
            tracing::error!("Failed to commit chained event transaction: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<SecurityEventFilter, SecurityEventSortField>,
    ) -> Result<Page<SecurityEvent>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            tracing::error!("Failed to list security events: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(Page::new(
            models.into_iter().map(Into::into).collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Unscoped<SecurityEvent>, CoreError>> + Send {
        let db = self.db.clone();
        async move {
            let model = security_events::Entity::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get security event by id: {}", e);
                    CoreError::InternalServerError
                })?
                .ok_or(CoreError::NotFound)?;

            Ok(Unscoped::new(model.into()))
        }
    }

    async fn get_events_ordered_for_verification(
        &self,
        realm_id: RealmId,
    ) -> Result<Vec<SecurityEvent>, CoreError> {
        let models = security_events::Entity::find()
            .filter(security_events::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .order_by_asc(security_events::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch events for chain verification: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::seawatch::{
        entities::{EventStatus, SecurityEventType},
        value_objects::SecurityEventFilter,
    };

    fn sql(filter: &SecurityEventFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&SecurityEventFilter::default());
        assert!(
            sql.ends_with(
                r#"WHERE "security_events"."realm_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn ip_address_is_an_escaped_contains_match() {
        let sql = sql(&SecurityEventFilter {
            ip_address: Some("10.%_".to_string()),
            ..SecurityEventFilter::default()
        });
        assert!(
            sql.contains(r#""security_events"."ip_address" ILIKE E'%10.\\%\\_%'"#),
            "{sql}"
        );
    }

    #[test]
    fn identifiers_status_and_target_type_are_exact_matches() {
        let sql = sql(&SecurityEventFilter {
            actor_id: Some(Uuid::max()),
            client_id: Some(Uuid::nil()),
            status: Some(EventStatus::Failure),
            target_type: Some("client".to_string()),
            ..SecurityEventFilter::default()
        });
        assert!(
            sql.contains(
                r#""security_events"."actor_id" = 'ffffffff-ffff-ffff-ffff-ffffffffffff'"#
            ),
            "{sql}"
        );
        assert!(
            sql.contains(
                r#""security_events"."target_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
        assert!(
            sql.contains(r#""security_events"."status" = 'failure'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""security_events"."target_type" = 'client'"#),
            "{sql}"
        );
        assert!(!sql.contains("ILIKE"), "{sql}");
    }

    #[test]
    fn event_types_are_an_in_list_of_wire_values() {
        let sql = sql(&SecurityEventFilter {
            event_types: Some(vec![
                SecurityEventType::LoginFailure,
                SecurityEventType::SessionRevoked,
            ]),
            ..SecurityEventFilter::default()
        });
        assert!(
            sql.contains(
                r#""security_events"."event_type" IN ('login_failure', 'session_revoked')"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn the_time_range_is_inclusive_on_timestamp() {
        let sql = sql(&SecurityEventFilter {
            from_timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 0, 5, 0).single(),
            to_timestamp: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).single(),
            ..SecurityEventFilter::default()
        });
        assert!(
            sql.contains(r#""security_events"."timestamp" >= '2026-01-01 00:05:00.000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""security_events"."timestamp" <= '2026-01-02 00:00:00.000000'"#),
            "{sql}"
        );
    }
}
