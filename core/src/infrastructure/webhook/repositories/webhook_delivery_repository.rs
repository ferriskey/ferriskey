use std::time::Duration;

use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbBackend, EntityTrait, QueryFilter, QueryTrait, Select,
    Statement,
};
use tracing::{error, warn};
use uuid::Uuid;

use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Unscoped};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::webhook::entities::retry_policy::{RetryPolicy, RetryPolicyOverride};
use crate::domain::webhook::entities::webhook::Webhook;
use crate::domain::webhook::entities::webhook_delivery::{
    DeliveryOutcome, DeliveryStatus, WebhookDelivery, WebhookDeliveryFilter, WebhookDeliveryId,
    WebhookDeliverySortField,
};
use crate::domain::webhook::ports::WebhookDeliveryRepository;
use crate::entity::realm_settings::{Column as RealmSettingsColumn, Entity as RealmSettingsEntity};
use crate::entity::webhook_deliveries::{
    ActiveModel as WebhookDeliveryActiveModel, Column as WebhookDeliveryColumn,
    Entity as WebhookDeliveryEntity,
};
use crate::entity::webhooks::{Column as WebhookColumn, Entity as WebhookEntity};
use crate::infrastructure::pagination::{SortColumn, paginate};

fn optional_u32(value: Option<i32>) -> Option<u32> {
    value.and_then(|value| u32::try_from(value).ok())
}

const CLAIM_DUE_SQL: &str = r#"
UPDATE webhook_deliveries
SET status = 'delivering', leased_until = $1, updated_at = $2
WHERE id IN (
    SELECT id
    FROM webhook_deliveries
    WHERE status = 'pending' AND next_attempt_at <= $2
    ORDER BY next_attempt_at
    LIMIT $3
    FOR UPDATE SKIP LOCKED
)
RETURNING *
"#;

#[derive(Debug, Clone)]
pub struct PostgresWebhookDeliveryRepository {
    pub db: DatabaseConnection,
}

impl PostgresWebhookDeliveryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn load(
        &self,
        realm_id: RealmId,
        id: WebhookDeliveryId,
    ) -> Result<WebhookDelivery, CoreError> {
        WebhookDeliveryEntity::find_by_id(id.as_uuid())
            .filter(WebhookDeliveryColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::WebhookDeliveryNotFound)
            .and_then(WebhookDelivery::try_from)
    }
}

fn to_i32(value: u32) -> Result<i32, CoreError> {
    i32::try_from(value).map_err(|_| CoreError::InternalServerError)
}

fn persisted_state(delivery: &WebhookDelivery) -> Result<WebhookDeliveryActiveModel, CoreError> {
    Ok(WebhookDeliveryActiveModel {
        status: Set(delivery.status.as_str().to_string()),
        attempt_count: Set(to_i32(delivery.attempt_count)?),
        next_attempt_at: Set(delivery.next_attempt_at.map(|at| at.naive_utc())),
        leased_until: Set(delivery.leased_until.map(|at| at.naive_utc())),
        last_attempt_at: Set(delivery.last_attempt_at.map(|at| at.naive_utc())),
        last_status_code: Set(delivery.last_status_code.map(i32::from)),
        last_error_code: Set(delivery.last_error_code.map(|code| code.as_code())),
        last_error_detail: Set(delivery.last_error_detail.clone()),
        updated_at: Set(delivery.updated_at.naive_utc()),
        ..Default::default()
    })
}

impl SortColumn<WebhookDeliveryEntity> for WebhookDeliverySortField {
    fn column(&self) -> WebhookDeliveryColumn {
        match self {
            WebhookDeliverySortField::Status => WebhookDeliveryColumn::Status,
            WebhookDeliverySortField::AttemptCount => WebhookDeliveryColumn::AttemptCount,
            WebhookDeliverySortField::LastAttemptAt => WebhookDeliveryColumn::LastAttemptAt,
            WebhookDeliverySortField::CreatedAt => WebhookDeliveryColumn::CreatedAt,
            WebhookDeliverySortField::UpdatedAt => WebhookDeliveryColumn::UpdatedAt,
        }
    }
}

fn listing_select(
    realm_id: Uuid,
    webhook_id: Uuid,
    filter: &WebhookDeliveryFilter,
) -> Select<WebhookDeliveryEntity> {
    WebhookDeliveryEntity::find()
        .filter(WebhookDeliveryColumn::RealmId.eq(realm_id))
        .filter(WebhookDeliveryColumn::WebhookId.eq(webhook_id))
        .apply_if(filter.event.as_ref(), |select, event| {
            select.filter(WebhookDeliveryColumn::Event.eq(event.to_string()))
        })
        .apply_if(filter.status, |select, status| {
            select.filter(WebhookDeliveryColumn::Status.eq(status.as_str()))
        })
        .apply_if(filter.resource_id, |select, resource_id| {
            select.filter(WebhookDeliveryColumn::ResourceId.eq(resource_id))
        })
}

impl WebhookDeliveryRepository for PostgresWebhookDeliveryRepository {
    async fn enqueue(&self, delivery: WebhookDelivery) -> Result<(), CoreError> {
        let state = persisted_state(&delivery)?;

        let model = WebhookDeliveryActiveModel {
            id: Set(delivery.id.as_uuid()),
            realm_id: Set(delivery.realm_id.into()),
            webhook_id: Set(delivery.webhook_id),
            event: Set(delivery.event.to_string()),
            resource_id: Set(delivery.resource_id),
            payload: Set(delivery.payload),
            created_at: Set(delivery.created_at.naive_utc()),
            ..state
        };

        WebhookDeliveryEntity::insert(model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to enqueue webhook delivery: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn claim_due(
        &self,
        limit: u32,
        lease: Duration,
    ) -> Result<Vec<WebhookDelivery>, CoreError> {
        let now = Utc::now().naive_utc();
        let lease =
            chrono::Duration::from_std(lease).map_err(|_| CoreError::InternalServerError)?;
        let leased_until = now + lease;

        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            CLAIM_DUE_SQL,
            [leased_until.into(), now.into(), i64::from(limit).into()],
        );

        WebhookDeliveryEntity::find()
            .from_raw_sql(statement)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to claim due webhook deliveries: {}", e);
                CoreError::InternalServerError
            })?
            .into_iter()
            .map(WebhookDelivery::try_from)
            .collect()
    }

    async fn record_outcome(
        &self,
        id: WebhookDeliveryId,
        outcome: DeliveryOutcome,
    ) -> Result<(), CoreError> {
        let model = WebhookDeliveryEntity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::WebhookDeliveryNotFound)?;

        let mut delivery = WebhookDelivery::try_from(model)?;
        delivery.apply(outcome);

        WebhookDeliveryEntity::update_many()
            .set(persisted_state(&delivery)?)
            .filter(WebhookDeliveryColumn::Id.eq(id.as_uuid()))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn reclaim_expired(&self, now: DateTime<Utc>) -> Result<u64, CoreError> {
        let result = WebhookDeliveryEntity::update_many()
            .set(WebhookDeliveryActiveModel {
                status: Set(DeliveryStatus::Pending.as_str().to_string()),
                leased_until: Set(None),
                next_attempt_at: Set(Some(now.naive_utc())),
                updated_at: Set(now.naive_utc()),
                ..Default::default()
            })
            .filter(WebhookDeliveryColumn::Status.eq(DeliveryStatus::Delivering.as_str()))
            .filter(WebhookDeliveryColumn::LeasedUntil.lt(now.naive_utc()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to reclaim expired webhook delivery leases: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }

    async fn list(
        &self,
        scope: &RealmScope,
        webhook: &Webhook,
        request: &PageRequest<WebhookDeliveryFilter, WebhookDeliverySortField>,
    ) -> Result<Page<WebhookDelivery>, CoreError> {
        let select = listing_select(scope.id().into(), webhook.id, &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("error listing webhook deliveries: {:?}", e);
            CoreError::InternalServerError
        })?;

        let deliveries = models
            .into_iter()
            .map(WebhookDelivery::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page::new(deliveries, total, request.page, request.limit))
    }

    async fn get(
        &self,
        realm_id: RealmId,
        id: WebhookDeliveryId,
    ) -> Result<Unscoped<WebhookDelivery>, CoreError> {
        self.load(realm_id, id).await.map(Unscoped::new)
    }

    async fn requeue(&self, realm_id: RealmId, id: WebhookDeliveryId) -> Result<(), CoreError> {
        let mut delivery = self.load(realm_id, id).await?;

        if !delivery.status.is_terminal() {
            return Err(CoreError::WebhookDeliveryNotReplayable);
        }

        delivery.requeue(Utc::now());

        WebhookDeliveryEntity::update_many()
            .set(persisted_state(&delivery)?)
            .filter(WebhookDeliveryColumn::Id.eq(id.as_uuid()))
            .filter(WebhookDeliveryColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn purge_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, CoreError> {
        let result = WebhookDeliveryEntity::delete_many()
            .filter(WebhookDeliveryColumn::CreatedAt.lt(cutoff.naive_utc()))
            .filter(WebhookDeliveryColumn::Status.is_in([
                DeliveryStatus::Succeeded.as_str(),
                DeliveryStatus::Failed.as_str(),
            ]))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to purge webhook deliveries: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }

    async fn resolve_retry_policy(
        &self,
        realm_id: RealmId,
        webhook_id: Uuid,
    ) -> Result<RetryPolicy, CoreError> {
        let webhook = WebhookEntity::find_by_id(webhook_id)
            .filter(WebhookColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let webhook_override = webhook
            .map(|webhook| RetryPolicyOverride {
                max_attempts: optional_u32(webhook.retry_max_attempts),
                base_delay_ms: optional_u32(webhook.retry_base_delay_ms),
                max_delay_ms: optional_u32(webhook.retry_max_delay_ms),
                max_total_delay_ms: optional_u32(webhook.retry_max_total_delay_ms),
            })
            .unwrap_or_default();

        let settings = RealmSettingsEntity::find()
            .filter(RealmSettingsColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let realm_override = settings
            .map(|settings| RetryPolicyOverride {
                max_attempts: optional_u32(settings.webhook_retry_max_attempts),
                base_delay_ms: optional_u32(settings.webhook_retry_base_delay_ms),
                max_delay_ms: optional_u32(settings.webhook_retry_max_delay_ms),
                max_total_delay_ms: optional_u32(settings.webhook_retry_max_total_delay_ms),
            })
            .unwrap_or_default();

        match RetryPolicy::resolve(webhook_override, realm_override) {
            Ok(policy) => Ok(policy),
            Err(error) => {
                warn!(
                    webhook_id = %webhook_id,
                    error = %error,
                    "stored webhook retry policy is out of bounds, falling back to the system default"
                );
                Ok(RetryPolicy::SYSTEM_DEFAULT)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::pagination::PageLimit;
    use crate::domain::realm::entities::{Realm, RealmScope};
    use crate::domain::webhook::entities::webhook_delivery::DeliveryErrorCode;
    use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
    use crate::infrastructure::seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository;
    use crate::infrastructure::webhook::delivery::{DeliveryJob, deliver_once};
    use ferriskey_webhook::endpoint::PrivateEndpoints;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    struct Fixture {
        repository: PostgresWebhookDeliveryRepository,
        realm_a: RealmId,
        realm_b: RealmId,
        webhook_a: Uuid,
        webhook_b: Uuid,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("webhook_delivery_test_{}", Uuid::new_v4().simple());

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

        let realm_a = Uuid::new_v4();
        let realm_b = Uuid::new_v4();
        for (id, label) in [(realm_a, "tenant-a"), (realm_b, "tenant-b")] {
            sqlx::query(
                "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
            )
            .bind(id)
            .bind(format!("{label}-{id}"))
            .execute(&pool)
            .await
            .expect("insert test realm");
        }

        let webhook_a = Uuid::new_v4();
        let webhook_b = Uuid::new_v4();
        for (id, realm) in [(webhook_a, realm_a), (webhook_b, realm_b)] {
            sqlx::query(
                "INSERT INTO webhooks (id, realm_id, endpoint, headers, secret, created_at, updated_at)
                 VALUES ($1, $2, 'https://example.test/hook', '{}'::jsonb, 'secret', NOW(), NOW())",
            )
            .bind(id)
            .bind(realm)
            .execute(&pool)
            .await
            .expect("insert test webhook");
        }

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        Fixture {
            repository: PostgresWebhookDeliveryRepository::new(db),
            realm_a: RealmId::from(realm_a),
            realm_b: RealmId::from(realm_b),
            webhook_a,
            webhook_b,
            pool,
        }
    }

    fn scope_of(realm_id: RealmId) -> RealmScope {
        let mut realm = Realm::new("fixture".to_string());
        realm.id = realm_id;
        RealmScope::from_realm(realm)
    }

    fn webhook_of(id: Uuid) -> Webhook {
        Webhook {
            id,
            endpoint: "https://example.test/hook".to_string(),
            headers: Default::default(),
            secret: String::new(),
            name: None,
            description: None,
            subscribers: Vec::new(),
            retry_policy: RetryPolicyOverride::default(),
            effective_retry_policy: None,
            triggered_at: None,
            updated_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    fn sample(realm_id: RealmId, webhook_id: Uuid) -> WebhookDelivery {
        WebhookDelivery::pending(
            realm_id,
            webhook_id,
            WebhookTrigger::UserCreated,
            Uuid::new_v4(),
            serde_json::json!({"event": "user.created"}),
            Utc::now(),
        )
    }

    async fn enqueue_due(fixture: &Fixture, count: usize) -> Vec<WebhookDeliveryId> {
        let mut ids = Vec::new();
        for _ in 0..count {
            let delivery = sample(fixture.realm_a, fixture.webhook_a);
            ids.push(delivery.id);
            fixture
                .repository
                .enqueue(delivery)
                .await
                .expect("enqueue delivery");
        }
        ids
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn concurrent_claims_never_hand_out_the_same_delivery_twice() {
        let fixture = setup().await;
        enqueue_due(&fixture, 10).await;

        let lease = Duration::from_secs(60);
        let (first, second) = tokio::join!(
            fixture.repository.claim_due(5, lease),
            fixture.repository.claim_due(5, lease)
        );

        let first = first.expect("first claim");
        let second = second.expect("second claim");

        let mut seen = std::collections::HashSet::new();
        for delivery in first.iter().chain(second.iter()) {
            assert!(
                seen.insert(delivery.id),
                "delivery {} was claimed twice",
                delivery.id
            );
        }

        assert_eq!(seen.len(), first.len() + second.len());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_row_locked_by_another_transaction_is_skipped_rather_than_waited_on() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 2).await;

        let mut locking_tx = fixture
            .pool
            .begin()
            .await
            .expect("begin locking transaction");
        sqlx::query("SELECT id FROM webhook_deliveries WHERE id = $1 FOR UPDATE")
            .bind(ids[0].as_uuid())
            .fetch_one(&mut *locking_tx)
            .await
            .expect("lock the first row");

        let claimed = tokio::time::timeout(
            Duration::from_secs(5),
            fixture.repository.claim_due(10, Duration::from_secs(60)),
        )
        .await
        .expect("claim_due blocked on a row locked by another transaction")
        .expect("claim");

        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].id, ids[1]);

        locking_tx
            .rollback()
            .await
            .expect("rollback locking transaction");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn claiming_marks_the_delivery_as_delivering_and_leases_it() {
        let fixture = setup().await;
        enqueue_due(&fixture, 1).await;

        let claimed = fixture
            .repository
            .claim_due(10, Duration::from_secs(60))
            .await
            .expect("claim");

        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].status, DeliveryStatus::Delivering);
        assert!(claimed[0].leased_until.is_some());

        let again = fixture
            .repository
            .claim_due(10, Duration::from_secs(60))
            .await
            .expect("second claim");

        assert!(again.is_empty(), "a leased delivery must not be re-claimed");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_delivery_due_in_the_future_is_never_claimed() {
        let fixture = setup().await;

        let mut delivery = sample(fixture.realm_a, fixture.webhook_a);
        delivery.next_attempt_at = Some(Utc::now() + chrono::Duration::hours(1));
        fixture
            .repository
            .enqueue(delivery)
            .await
            .expect("enqueue future delivery");

        let claimed = fixture
            .repository
            .claim_due(10, Duration::from_secs(60))
            .await
            .expect("claim");

        assert!(claimed.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_expired_lease_is_reclaimed_and_becomes_claimable_again() {
        let fixture = setup().await;

        let mut delivery = sample(fixture.realm_a, fixture.webhook_a);
        delivery.status = DeliveryStatus::Delivering;
        delivery.leased_until = Some(Utc::now() - chrono::Duration::minutes(10));
        delivery.next_attempt_at = None;
        fixture
            .repository
            .enqueue(delivery)
            .await
            .expect("enqueue stuck delivery");

        let reclaimed = fixture
            .repository
            .reclaim_expired(Utc::now())
            .await
            .expect("reclaim");

        assert_eq!(reclaimed, 1);

        let claimed = fixture
            .repository
            .claim_due(10, Duration::from_secs(60))
            .await
            .expect("claim after reclaim");

        assert_eq!(claimed.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_live_lease_is_left_alone() {
        let fixture = setup().await;
        enqueue_due(&fixture, 1).await;

        fixture
            .repository
            .claim_due(10, Duration::from_secs(600))
            .await
            .expect("claim");

        let reclaimed = fixture
            .repository
            .reclaim_expired(Utc::now())
            .await
            .expect("reclaim");

        assert_eq!(reclaimed, 0);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn deleting_a_webhook_cascades_its_deliveries_away() {
        let fixture = setup().await;
        enqueue_due(&fixture, 3).await;

        sqlx::query("DELETE FROM webhooks WHERE id = $1")
            .bind(fixture.webhook_a)
            .execute(&fixture.pool)
            .await
            .expect("delete webhook");

        let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM webhook_deliveries")
            .fetch_one(&fixture.pool)
            .await
            .expect("count deliveries");

        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_delivery_is_invisible_to_another_realm() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 1).await;

        let foreign = fixture.repository.get(fixture.realm_b, ids[0]).await;
        assert!(
            matches!(foreign, Err(CoreError::WebhookDeliveryNotFound)),
            "expected WebhookDeliveryNotFound, got {foreign:?}"
        );

        let owned = fixture.repository.get(fixture.realm_a, ids[0]).await;
        assert!(owned.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn listing_is_scoped_to_its_realm_and_webhook() {
        let fixture = setup().await;
        enqueue_due(&fixture, 2).await;

        let foreign = fixture
            .repository
            .list(
                &scope_of(fixture.realm_b),
                &webhook_of(fixture.webhook_b),
                &PageRequest::default(),
            )
            .await
            .expect("list foreign");

        assert_eq!(foreign.metadata().total, 0);
        assert!(foreign.data().is_empty());

        let crossed = fixture
            .repository
            .list(
                &scope_of(fixture.realm_b),
                &webhook_of(fixture.webhook_a),
                &PageRequest::default(),
            )
            .await
            .expect("list crossed");

        assert_eq!(crossed.metadata().total, 0);
        assert!(crossed.data().is_empty());

        let owned = fixture
            .repository
            .list(
                &scope_of(fixture.realm_a),
                &webhook_of(fixture.webhook_a),
                &PageRequest::default(),
            )
            .await
            .expect("list owned");

        assert_eq!(owned.metadata().total, 2);
        assert_eq!(owned.data().len(), 2);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn listing_reports_the_total_beyond_the_requested_page() {
        let fixture = setup().await;
        enqueue_due(&fixture, 5).await;

        let request = PageRequest {
            limit: PageLimit::try_from(2).expect("valid limit"),
            ..PageRequest::default()
        };
        let page = fixture
            .repository
            .list(
                &scope_of(fixture.realm_a),
                &webhook_of(fixture.webhook_a),
                &request,
            )
            .await
            .expect("list page");

        assert_eq!(page.data().len(), 2);
        assert_eq!(page.metadata().total, 5);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn recording_an_outcome_advances_the_attempt_count_and_state() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 1).await;

        fixture
            .repository
            .record_outcome(
                ids[0],
                DeliveryOutcome::Exhausted {
                    at: Utc::now(),
                    error: DeliveryErrorCode::HttpStatus(503),
                    detail: Some("service unavailable".to_string()),
                },
            )
            .await
            .expect("record outcome");

        let stored = fixture
            .repository
            .get(fixture.realm_a, ids[0])
            .await
            .expect("reload delivery")
            .in_realm(&scope_of(fixture.realm_a))
            .expect("the reloaded delivery belongs to the realm it was read through")
            .into_inner();

        assert_eq!(stored.status, DeliveryStatus::Failed);
        assert_eq!(stored.attempt_count, 1);
        assert_eq!(stored.next_attempt_at, None);
        assert_eq!(stored.last_status_code, Some(503));
        assert_eq!(
            stored.last_error_code,
            Some(DeliveryErrorCode::HttpStatus(503))
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_failed_delivery_can_be_replayed() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 1).await;

        fixture
            .repository
            .record_outcome(
                ids[0],
                DeliveryOutcome::Exhausted {
                    at: Utc::now(),
                    error: DeliveryErrorCode::Transport,
                    detail: None,
                },
            )
            .await
            .expect("record outcome");

        fixture
            .repository
            .requeue(fixture.realm_a, ids[0])
            .await
            .expect("requeue");

        let stored = fixture
            .repository
            .get(fixture.realm_a, ids[0])
            .await
            .expect("reload delivery")
            .in_realm(&scope_of(fixture.realm_a))
            .expect("the reloaded delivery belongs to the realm it was read through")
            .into_inner();

        assert_eq!(stored.status, DeliveryStatus::Pending);
        assert_eq!(stored.attempt_count, 0);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_in_flight_delivery_cannot_be_replayed() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 1).await;

        let result = fixture.repository.requeue(fixture.realm_a, ids[0]).await;

        assert!(
            matches!(result, Err(CoreError::WebhookDeliveryNotReplayable)),
            "expected WebhookDeliveryNotReplayable, got {result:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn replaying_across_realms_is_refused() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 1).await;

        let result = fixture.repository.requeue(fixture.realm_b, ids[0]).await;

        assert!(
            matches!(result, Err(CoreError::WebhookDeliveryNotFound)),
            "expected WebhookDeliveryNotFound, got {result:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn purging_removes_terminal_rows_and_spares_the_rest() {
        let fixture = setup().await;

        let mut succeeded = sample(fixture.realm_a, fixture.webhook_a);
        succeeded.status = DeliveryStatus::Succeeded;
        succeeded.created_at = Utc::now() - chrono::Duration::days(60);
        succeeded.next_attempt_at = None;

        let mut failed = sample(fixture.realm_a, fixture.webhook_a);
        failed.status = DeliveryStatus::Failed;
        failed.created_at = Utc::now() - chrono::Duration::days(60);
        failed.next_attempt_at = None;

        let mut pending = sample(fixture.realm_a, fixture.webhook_a);
        pending.created_at = Utc::now() - chrono::Duration::days(60);

        for delivery in [succeeded, failed, pending] {
            fixture
                .repository
                .enqueue(delivery)
                .await
                .expect("enqueue delivery");
        }

        let purged = fixture
            .repository
            .purge_older_than(Utc::now() - chrono::Duration::days(30))
            .await
            .expect("purge");

        assert_eq!(purged, 2);

        let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM webhook_deliveries")
            .fetch_one(&fixture.pool)
            .await
            .expect("count deliveries");

        assert_eq!(remaining, 1);
    }

    fn job_for(fixture: &Fixture, id: WebhookDeliveryId, endpoint: &str) -> DeliveryJob {
        DeliveryJob {
            delivery_id: id,
            realm_id: fixture.realm_a,
            webhook_id: fixture.webhook_a,
            event: WebhookTrigger::UserCreated,
            endpoint: endpoint.to_string(),
            headers: std::collections::HashMap::new(),
            secret: "test-secret".to_string(),
            body: std::sync::Arc::new(b"{}".to_vec()),
            attempt_count: 0,
            elapsed: Duration::ZERO,
            policy: RetryPolicy::SYSTEM_DEFAULT,
            private_endpoints: PrivateEndpoints::Forbidden,
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_transient_failure_schedules_another_attempt_instead_of_giving_up() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 1).await;
        let security = PostgresSecurityEventRepository::new(fixture.repository.db.clone());

        let endpoint = format!(
            "https://nonexistent-{}.invalid/hook",
            Uuid::new_v4().simple()
        );
        deliver_once(
            job_for(&fixture, ids[0], &endpoint),
            &fixture.repository,
            &security,
        )
        .await;

        let stored = fixture
            .repository
            .get(fixture.realm_a, ids[0])
            .await
            .expect("reload delivery")
            .in_realm(&scope_of(fixture.realm_a))
            .expect("the reloaded delivery belongs to the realm it was read through")
            .into_inner();

        assert_eq!(stored.status, DeliveryStatus::Pending);
        assert_eq!(stored.attempt_count, 1);
        assert!(stored.last_error_code.is_some());

        let next = stored.next_attempt_at.expect("a retry must be scheduled");
        assert!(
            next >= stored.created_at,
            "a retry must not be scheduled before the delivery existed"
        );
        assert!(
            next <= Utc::now() + chrono::Duration::seconds(31),
            "a retry must stay within the backoff ceiling"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_blocked_address_exhausts_immediately_and_is_audited() {
        let fixture = setup().await;
        let ids = enqueue_due(&fixture, 1).await;
        let security = PostgresSecurityEventRepository::new(fixture.repository.db.clone());

        deliver_once(
            job_for(&fixture, ids[0], "http://127.0.0.1:1/hook"),
            &fixture.repository,
            &security,
        )
        .await;

        let stored = fixture
            .repository
            .get(fixture.realm_a, ids[0])
            .await
            .expect("reload delivery")
            .in_realm(&scope_of(fixture.realm_a))
            .expect("the reloaded delivery belongs to the realm it was read through")
            .into_inner();

        assert_eq!(stored.status, DeliveryStatus::Failed);
        assert_eq!(stored.next_attempt_at, None);
        assert_eq!(
            stored.last_error_code,
            Some(DeliveryErrorCode::NoUsableAddress)
        );

        let audited: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM security_events WHERE event_type = 'webhook_delivery_exhausted'",
        )
        .fetch_one(&fixture.pool)
        .await
        .expect("count security events");

        assert_eq!(audited, 1, "exhaustion must leave exactly one audit event");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_webhook_without_any_override_inherits_the_system_default() {
        let fixture = setup().await;

        let policy = fixture
            .repository
            .resolve_retry_policy(fixture.realm_a, fixture.webhook_a)
            .await
            .expect("resolve policy");

        assert_eq!(policy, RetryPolicy::SYSTEM_DEFAULT);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_partial_webhook_override_keeps_the_other_fields_at_their_default() {
        let fixture = setup().await;

        sqlx::query("UPDATE webhooks SET retry_max_attempts = 12 WHERE id = $1")
            .bind(fixture.webhook_a)
            .execute(&fixture.pool)
            .await
            .expect("set webhook override");

        let policy = fixture
            .repository
            .resolve_retry_policy(fixture.realm_a, fixture.webhook_a)
            .await
            .expect("resolve policy");

        assert_eq!(policy.max_attempts(), 12);
        assert_eq!(
            policy.base_delay(),
            RetryPolicy::SYSTEM_DEFAULT.base_delay()
        );
        assert_eq!(policy.max_delay(), RetryPolicy::SYSTEM_DEFAULT.max_delay());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_out_of_bounds_stored_override_falls_back_to_the_system_default() {
        let fixture = setup().await;

        sqlx::query("UPDATE webhooks SET retry_max_attempts = 0 WHERE id = $1")
            .bind(fixture.webhook_a)
            .execute(&fixture.pool)
            .await
            .expect("set invalid override");

        let policy = fixture
            .repository
            .resolve_retry_policy(fixture.realm_a, fixture.webhook_a)
            .await
            .expect("an invalid stored policy must not fail the delivery");

        assert_eq!(policy, RetryPolicy::SYSTEM_DEFAULT);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_recent_terminal_row_survives_the_purge() {
        let fixture = setup().await;

        let mut succeeded = sample(fixture.realm_a, fixture.webhook_a);
        succeeded.status = DeliveryStatus::Succeeded;
        succeeded.next_attempt_at = None;
        fixture
            .repository
            .enqueue(succeeded)
            .await
            .expect("enqueue delivery");

        let purged = fixture
            .repository
            .purge_older_than(Utc::now() - chrono::Duration::days(30))
            .await
            .expect("purge");

        assert_eq!(purged, 0);
    }
}

#[cfg(test)]
mod listing_tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::webhook::entities::webhook_delivery::{
        DeliveryStatus, WebhookDeliveryFilter,
    };
    use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;

    const REALM: Uuid = Uuid::from_u128(1);
    const WEBHOOK: Uuid = Uuid::from_u128(2);

    fn sql(filter: &WebhookDeliveryFilter) -> String {
        listing_select(REALM, WEBHOOK, filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm_and_the_webhook() {
        let sql = sql(&WebhookDeliveryFilter::default());
        assert!(
            sql.contains(&format!(r#""webhook_deliveries"."realm_id" = '{REALM}'"#)),
            "{sql}"
        );
        assert!(
            sql.contains(&format!(
                r#""webhook_deliveries"."webhook_id" = '{WEBHOOK}'"#
            )),
            "{sql}"
        );
    }

    #[test]
    fn every_filter_is_an_equality() {
        let resource = Uuid::from_u128(3);
        let sql = sql(&WebhookDeliveryFilter {
            event: Some(WebhookTrigger::UserCreated),
            status: Some(DeliveryStatus::Failed),
            resource_id: Some(resource),
        });
        assert!(
            sql.contains(r#""webhook_deliveries"."event" = 'user.created'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""webhook_deliveries"."status" = 'failed'"#),
            "{sql}"
        );
        assert!(
            sql.contains(&format!(
                r#""webhook_deliveries"."resource_id" = '{resource}'"#
            )),
            "{sql}"
        );
        assert!(!sql.contains("LIKE"), "{sql}");
    }

    #[test]
    fn absent_filters_add_no_condition() {
        let sql = sql(&WebhookDeliveryFilter::default());
        let conditions = sql.split(" WHERE ").nth(1).expect("a WHERE clause");
        assert!(!conditions.contains(r#""event""#), "{sql}");
        assert!(!conditions.contains(r#""status""#), "{sql}");
        assert!(!conditions.contains(r#""resource_id""#), "{sql}");
    }
}
