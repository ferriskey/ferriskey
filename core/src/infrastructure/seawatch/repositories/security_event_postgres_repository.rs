use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::scope::RealmScope;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, QueryFilter,
    QueryOrder, QueryTrait, Select, Statement, Value,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::{RealmId, Unscoped};
use crate::domain::seawatch::{
    entities::SecurityEvent,
    hashing::{GENESIS_PREV_HASH, canonical_preimage},
    pii::{AuditPiiMode, PiiConfig, apply_to_event},
    ports::SecurityEventRepository,
    value_objects::{SecurityEventFilter, SecurityEventSortField},
};
use crate::entity::security_events;
use crate::infrastructure::pagination::{SortColumn, contains, paginate};

const PII_CONFIG_TTL: Duration = Duration::from_secs(30);
const CHAIN_ATTEMPTS: usize = 2;

const ENSURE_HEAD_SQL: &str = r#"
INSERT INTO security_event_chain_heads (realm_id, event_hash, prev_hash)
SELECT $1,
       (SELECT event_hash FROM security_events WHERE realm_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1),
       (SELECT prev_hash FROM security_events WHERE realm_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1)
ON CONFLICT (realm_id) DO NOTHING
"#;

const CHAINED_INSERT_SQL: &str = r#"
WITH head AS (
    UPDATE security_event_chain_heads
    SET prev_hash = event_hash,
        event_hash = encode(sha256($15 || COALESCE(decode(event_hash, 'hex'), $16)), 'hex'),
        updated_at = NOW()
    WHERE realm_id = $2
    RETURNING event_hash, prev_hash
)
INSERT INTO security_events
    (id, realm_id, actor_id, actor_type, event_type, status, target_type, target_id, resource,
     "timestamp", trace_id, ip_address, user_agent, details, event_hash, prev_hash, created_at)
SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
       head.event_hash, COALESCE(head.prev_hash, encode($16, 'hex')), clock_timestamp()
FROM head
RETURNING id
"#;

fn chained_insert(event: &SecurityEvent, preimage: Vec<u8>) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        CHAINED_INSERT_SQL,
        [
            event.id.0.into(),
            Uuid::from(event.realm_id).into(),
            event.actor_id.into(),
            event.actor_type.as_ref().map(ToString::to_string).into(),
            event.event_type.to_string().into(),
            event.status.to_string().into(),
            event.target_type.clone().into(),
            event.target_id.into(),
            event.resource.clone().into(),
            event.timestamp.naive_utc().into(),
            event.trace_id.clone().into(),
            event.ip_address.clone().into(),
            event.user_agent.clone().into(),
            Value::Json(event.details.clone().map(Box::new)),
            preimage.into(),
            GENESIS_PREV_HASH.to_vec().into(),
        ],
    )
}

#[derive(Debug, Clone)]
pub struct PostgresSecurityEventRepository {
    pub db: DatabaseConnection,
    pii_configs: Arc<Mutex<HashMap<Uuid, (PiiConfig, Instant)>>>,
}

impl PostgresSecurityEventRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            pii_configs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn pii_config(&self, realm_id: Uuid) -> PiiConfig {
        let cached = self
            .pii_configs
            .lock()
            .ok()
            .and_then(|configs| configs.get(&realm_id).cloned());
        if let Some((config, loaded_at)) = cached
            && loaded_at.elapsed() < PII_CONFIG_TTL
        {
            return config;
        }

        let config = self.load_pii_config(realm_id).await;
        if let Ok(mut configs) = self.pii_configs.lock() {
            configs.insert(realm_id, (config.clone(), Instant::now()));
        }
        config
    }

    async fn ensure_head(&self, realm_id: Uuid) -> Result<(), CoreError> {
        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                ENSURE_HEAD_SQL,
                [realm_id.into()],
            ))
            .await
            .map_err(|e| {
                error!("Failed to initialise the audit chain head: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
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
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(contains(Column::IpAddress, value))
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
    /// Advances the realm's head row and inserts the event in one statement:
    /// the row lock serialises writers and Postgres computes
    /// `event_hash = SHA-256(preimage || prev_hash)` from the head it just
    /// locked, so concurrent writers chain linearly instead of forking.
    async fn store_event(&self, event: SecurityEvent) -> Result<(), CoreError> {
        let realm_id: Uuid = event.realm_id.into();
        let pii_cfg = self.pii_config(realm_id).await;
        let event = apply_to_event(event, &pii_cfg);
        let preimage = canonical_preimage(&event).into_bytes();

        for _ in 0..CHAIN_ATTEMPTS {
            let inserted = self
                .db
                .query_one(chained_insert(&event, preimage.clone()))
                .await
                .map_err(|e| {
                    error!("Failed to store chained security event: {}", e);
                    CoreError::InternalServerError
                })?;

            if inserted.is_some() {
                return Ok(());
            }

            self.ensure_head(realm_id).await?;
        }

        error!(%realm_id, "The audit chain head could not be initialised");
        Err(CoreError::InternalServerError)
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
    fn search_is_an_escaped_contains_match_on_the_ip_address() {
        let sql = sql(&SecurityEventFilter {
            search: Some("a%_\\".to_string()),
            ..SecurityEventFilter::default()
        });
        assert!(
            sql.contains(r#""security_events"."ip_address" ILIKE E'%a\\%\\_\\\\%'"#),
            "{sql}"
        );
        assert!(!sql.contains(r#""user_agent" ILIKE"#), "{sql}");
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

    mod postgres {
        use sea_orm::Database as SeaOrmDatabase;
        use sqlx::Executor as _;
        use uuid::Uuid;

        use crate::domain::realm::entities::RealmId;
        use crate::domain::seawatch::entities::{EventStatus, SecurityEvent, SecurityEventType};
        use crate::domain::seawatch::hashing::{GENESIS_PREV_HASH, VerifyResult, verify_chain};
        use crate::domain::seawatch::ports::SecurityEventRepository;
        use crate::infrastructure::seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository;

        struct Fixture {
            repository: PostgresSecurityEventRepository,
            db: sea_orm::DatabaseConnection,
            pool: sqlx::PgPool,
        }

        async fn setup() -> Fixture {
            let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
            });

            let schema = format!("security_event_repository_test_{}", Uuid::new_v4().simple());

            let admin_pool = sqlx::PgPool::connect(&base_url)
                .await
                .expect("connect admin pool");
            admin_pool
                .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{}""#, schema)))
                .await
                .expect("create test schema");

            let separator = if base_url.contains('?') { '&' } else { '?' };
            let schema_url = format!("{base_url}{separator}options=-c search_path={schema}");
            let pool = sqlx::PgPool::connect(&schema_url)
                .await
                .expect("connect schema pool");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("run migrations");

            let db = SeaOrmDatabase::connect(&schema_url)
                .await
                .expect("sea-orm connect");

            Fixture {
                repository: PostgresSecurityEventRepository::new(db.clone()),
                db,
                pool,
            }
        }

        async fn insert_realm(pool: &sqlx::PgPool) -> RealmId {
            let realm_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
            )
            .bind(realm_id)
            .bind(format!("realm-{}", realm_id.simple()))
            .execute(pool)
            .await
            .expect("insert realm");

            RealmId::from(realm_id)
        }

        fn login_event(realm_id: RealmId, index: usize) -> SecurityEvent {
            SecurityEvent::new(
                realm_id,
                SecurityEventType::LoginSuccess,
                EventStatus::Success,
                Uuid::new_v4(),
            )
            .with_details(serde_json::json!({ "index": index }))
        }

        async fn stored_head(pool: &sqlx::PgPool, realm_id: RealmId) -> Option<String> {
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT event_hash FROM security_event_chain_heads WHERE realm_id = $1",
            )
            .bind(Uuid::from(realm_id))
            .fetch_one(pool)
            .await
            .expect("read the chain head")
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
        async fn concurrent_events_form_one_linear_chain() {
            let fixture = setup().await;
            let realm_id = insert_realm(&fixture.pool).await;
            let separate_cache = PostgresSecurityEventRepository::new(fixture.db.clone());

            let mut tasks = tokio::task::JoinSet::new();
            for index in 0..40 {
                let repository = if index % 3 == 0 {
                    separate_cache.clone()
                } else {
                    fixture.repository.clone()
                };
                tasks.spawn(
                    async move { repository.store_event(login_event(realm_id, index)).await },
                );
            }
            while let Some(result) = tasks.join_next().await {
                result.expect("task").expect("store_event");
            }

            let events = fixture
                .repository
                .get_events_ordered_for_verification(realm_id)
                .await
                .expect("events");
            assert_eq!(events.len(), 40);
            assert_eq!(verify_chain(&events), VerifyResult::Ok);

            let last_hash = events
                .last()
                .and_then(|event| event.event_hash)
                .map(hex::encode);
            assert_eq!(stored_head(&fixture.pool, realm_id).await, last_hash);
        }

        #[tokio::test]
        #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
        async fn the_first_event_of_a_realm_chains_on_genesis() {
            let fixture = setup().await;
            let realm_id = insert_realm(&fixture.pool).await;

            fixture
                .repository
                .store_event(login_event(realm_id, 0))
                .await
                .expect("store_event");

            let events = fixture
                .repository
                .get_events_ordered_for_verification(realm_id)
                .await
                .expect("events");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].prev_hash, Some(GENESIS_PREV_HASH));
            assert_eq!(
                stored_head(&fixture.pool, realm_id).await,
                events[0].event_hash.map(hex::encode)
            );
        }

        #[tokio::test]
        #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
        async fn two_repository_instances_interleave_on_one_linear_chain() {
            let fixture = setup().await;
            let realm_id = insert_realm(&fixture.pool).await;
            let other = PostgresSecurityEventRepository::new(fixture.db.clone());

            fixture
                .repository
                .store_event(login_event(realm_id, 0))
                .await
                .expect("first event");
            other
                .store_event(login_event(realm_id, 1))
                .await
                .expect("second event from another process");
            fixture
                .repository
                .store_event(login_event(realm_id, 2))
                .await
                .expect("third event with a stale cached head");

            let events = fixture
                .repository
                .get_events_ordered_for_verification(realm_id)
                .await
                .expect("events");
            assert_eq!(events.len(), 3);
            assert_eq!(verify_chain(&events), VerifyResult::Ok);
        }

        #[tokio::test]
        #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
        async fn a_realm_whose_last_event_predates_the_chain_restarts_at_genesis() {
            let fixture = setup().await;
            let realm_id = insert_realm(&fixture.pool).await;
            sqlx::query(
                "INSERT INTO security_events (id, realm_id, event_type, status, timestamp) \
                 VALUES ($1, $2, 'login_success', 'success', NOW() - INTERVAL '1 day')",
            )
            .bind(Uuid::new_v4())
            .bind(Uuid::from(realm_id))
            .execute(&fixture.pool)
            .await
            .expect("insert a legacy event without hashes");

            fixture
                .repository
                .store_event(login_event(realm_id, 0))
                .await
                .expect("store_event");

            let events = fixture
                .repository
                .get_events_ordered_for_verification(realm_id)
                .await
                .expect("events");
            assert_eq!(events.len(), 2);
            assert_eq!(events[1].prev_hash, Some(GENESIS_PREV_HASH));
            assert_eq!(
                stored_head(&fixture.pool, realm_id).await,
                events[1].event_hash.map(hex::encode)
            );
        }
    }
}
