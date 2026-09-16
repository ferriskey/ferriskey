use std::collections::HashMap;

use ferriskey_domain::realm::RealmId;
use ferriskey_webhook::signing::generate_secret;
use serde::Serialize;
use serde_json::to_value;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError,
    webhook::{
        entities::{
            webhook::Webhook, webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger,
        },
        ports::WebhookRepository,
    },
};

use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, RelationTrait,
};
use tokio::sync::mpsc;
use tracing::{error, warn};

use crate::domain::common::generate_timestamp;
use crate::domain::webhook::entities::retry_policy::{RetryPolicy, RetryPolicyOverride};
use crate::domain::webhook::entities::webhook_delivery::WebhookDelivery;
use crate::domain::webhook::entities::webhook_subscriber::WebhookSubscriber;
use crate::domain::webhook::ports::WebhookDeliveryRepository;
use crate::entity::webhook_subscribers::{
    ActiveModel as WebhookSubscriberActiveModel, Column as WebhookSubscriberColumn,
    Entity as WebhookSubscriberEntity,
};
use crate::entity::webhooks::{
    ActiveModel as WebhookActiveModel, Column as WebhookColumn, Entity as WebhookEntity,
    Relation as WebhookRelation,
};
use crate::infrastructure::seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository;
use crate::infrastructure::webhook::repositories::webhook_delivery_repository::PostgresWebhookDeliveryRepository;

use crate::entity::webhook_subscribers::Model as WebhookSubscriberModel;
use crate::infrastructure::webhook::delivery::{self, DeliveryJob};

fn as_column(value: Option<u32>) -> Option<i32> {
    value.and_then(|value| i32::try_from(value).ok())
}

fn column_update(
    policy: Option<RetryPolicyOverride>,
    field: impl Fn(RetryPolicyOverride) -> Option<u32>,
) -> sea_orm::ActiveValue<Option<i32>> {
    match policy {
        Some(policy) => Set(as_column(field(policy))),
        None => sea_orm::ActiveValue::NotSet,
    }
}

#[derive(Debug, Clone)]
pub struct PostgresWebhookRepository {
    pub db: DatabaseConnection,
    deliveries: PostgresWebhookDeliveryRepository,
    delivery_sender: mpsc::Sender<DeliveryJob>,
}

impl PostgresWebhookRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        let deliveries = PostgresWebhookDeliveryRepository::new(db.clone());
        let security_events = PostgresSecurityEventRepository::new(db.clone());
        let delivery_sender = delivery::spawn_dispatcher(deliveries.clone(), security_events);

        Self {
            db,
            deliveries,
            delivery_sender,
        }
    }
}

impl WebhookRepository for PostgresWebhookRepository {
    async fn fetch_webhooks_by_realm(&self, realm_id: RealmId) -> Result<Vec<Webhook>, CoreError> {
        let webhooks = WebhookEntity::find()
            .filter(WebhookColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .iter()
            .map(Webhook::from)
            .collect::<Vec<Webhook>>();

        Ok(webhooks)
    }

    async fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: RealmId,
        subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, CoreError> {
        let webhooks = WebhookEntity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                WebhookRelation::WebhookSubscribers.def(),
            )
            .filter(WebhookColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(WebhookSubscriberColumn::Name.eq(subscriber.to_string()))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch webhooks by subscriber: {}", e);
                CoreError::InternalServerError
            })?
            .into_iter()
            .map(Webhook::from)
            .collect();

        Ok(webhooks)
    }

    async fn get_webhook_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: RealmId,
    ) -> Result<Option<Webhook>, CoreError> {
        let mut webhook = WebhookEntity::find()
            .filter(WebhookColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(WebhookColumn::Id.eq(webhook_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .map(Webhook::from);

        if let Some(ref mut webhook) = webhook {
            let subscribers = WebhookSubscriberEntity::find()
                .filter(WebhookSubscriberColumn::WebhookId.eq(webhook_id))
                .all(&self.db)
                .await
                .map_err(|_| CoreError::InternalServerError)?
                .into_iter()
                .map(|s| s.try_into())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| CoreError::InternalServerError)?;

            webhook.subscribers = subscribers;
        }

        Ok(webhook)
    }

    async fn create_webhook(
        &self,
        realm_id: RealmId,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        headers: HashMap<String, String>,
        subscribers: Vec<WebhookTrigger>,
        retry_policy: RetryPolicyOverride,
    ) -> Result<Webhook, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let subscription_id = Uuid::new_v7(timestamp);
        let headers_json = to_value(headers).unwrap_or_default();

        let mut webhook = WebhookEntity::insert(WebhookActiveModel {
            id: Set(subscription_id),
            endpoint: Set(endpoint),
            headers: Set(headers_json),
            secret: Set(generate_secret()),
            name: Set(name),
            description: Set(description),
            realm_id: Set(realm_id.into()),
            triggered_at: Set(None),
            last_delivery_status: Set(None),
            last_delivery_error: Set(None),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            retry_max_attempts: Set(as_column(retry_policy.max_attempts)),
            retry_base_delay_ms: Set(as_column(retry_policy.base_delay_ms)),
            retry_max_delay_ms: Set(as_column(retry_policy.max_delay_ms)),
            retry_max_total_delay_ms: Set(as_column(retry_policy.max_total_delay_ms)),
        })
        .exec_with_returning(&self.db)
        .await
        .map(Webhook::from)
        .map_err(|e| {
            error!("Failed to create webhook: {}", e);
            CoreError::InternalServerError
        })?;

        let subscribers: Vec<WebhookSubscriber> = if subscribers.is_empty() {
            Vec::new()
        } else {
            let subscribers_model: Vec<WebhookSubscriberModel> =
                WebhookSubscriberEntity::insert_many(subscribers.iter().map(|value| {
                    WebhookSubscriberActiveModel {
                        id: Set(Uuid::new_v7(timestamp)),
                        name: Set(value.to_string()),
                        webhook_id: Set(subscription_id),
                    }
                }))
                .exec_with_returning_many(&self.db)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            subscribers_model
                .iter()
                .map(|value| value.clone().try_into())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| CoreError::InternalServerError)?
        };

        webhook.subscribers = subscribers;
        Ok(webhook)
    }

    async fn update_webhook(
        &self,
        realm_id: RealmId,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        headers: Option<HashMap<String, String>>,
        subscribers: Vec<WebhookTrigger>,
        retry_policy: Option<RetryPolicyOverride>,
    ) -> Result<Webhook, CoreError> {
        let update_result = WebhookEntity::update_many()
            .set(WebhookActiveModel {
                name: Set(name),
                description: Set(description),
                endpoint: Set(endpoint),
                // `NotSet` leaves the column alone — the caller said nothing
                // about headers, so the stored ones stay.
                headers: match headers {
                    Some(headers) => Set(to_value(headers).unwrap_or_default()),
                    None => sea_orm::ActiveValue::NotSet,
                },
                updated_at: Set(Utc::now().naive_utc()),
                retry_max_attempts: column_update(retry_policy, |p| p.max_attempts),
                retry_base_delay_ms: column_update(retry_policy, |p| p.base_delay_ms),
                retry_max_delay_ms: column_update(retry_policy, |p| p.max_delay_ms),
                retry_max_total_delay_ms: column_update(retry_policy, |p| p.max_total_delay_ms),
                ..Default::default()
            })
            .filter(WebhookColumn::Id.eq(id))
            .filter(WebhookColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if update_result.rows_affected == 0 {
            return Err(CoreError::WebhookNotFound);
        }

        WebhookSubscriberEntity::delete_many()
            .filter(WebhookSubscriberColumn::WebhookId.eq(id))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut derived_subscribers = Vec::new();
        for subscriber in subscribers {
            let (_, timestamp) = generate_timestamp();

            let subscription_id = Uuid::new_v7(timestamp);
            let subscriber = WebhookSubscriberActiveModel {
                id: Set(subscription_id),
                name: Set(subscriber.to_string()),
                webhook_id: Set(id),
            };

            derived_subscribers.push(subscriber);
        }

        let subscribers = if derived_subscribers.is_empty() {
            Vec::new()
        } else {
            WebhookSubscriberEntity::insert_many(derived_subscribers)
                .exec_with_returning_many(&self.db)
                .await
                .map_err(|_| CoreError::InternalServerError)?
                .iter()
                .map(|value| value.clone().try_into())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| CoreError::InternalServerError)?
        };

        let mut webhook = WebhookEntity::find()
            .filter(WebhookColumn::Id.eq(id))
            .filter(WebhookColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .map(Webhook::from)
            .ok_or(CoreError::WebhookNotFound)?;

        webhook.subscribers = subscribers;

        Ok(webhook)
    }

    async fn delete_webhook(&self, realm_id: RealmId, id: Uuid) -> Result<(), CoreError> {
        let delete_result = WebhookEntity::delete_many()
            .filter(WebhookColumn::Id.eq(id))
            .filter(WebhookColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if delete_result.rows_affected == 0 {
            return Err(CoreError::WebhookNotFound);
        }

        Ok(())
    }

    async fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        realm_id: RealmId,
        payload: WebhookPayload<T>,
    ) -> Result<(), CoreError> {
        let webhooks = match self
            .fetch_webhooks_by_subscriber(realm_id, payload.event.clone())
            .await
        {
            Ok(webhooks) => webhooks,
            Err(err) => {
                error!("Failed to fetch webhooks: {:?}", err);
                return Ok(());
            }
        };

        if webhooks.is_empty() {
            return Ok(());
        }

        let document = match to_value(&payload) {
            Ok(document) => document,
            Err(err) => {
                error!("Failed to serialize webhook payload: {:?}", err);
                return Ok(());
            }
        };

        let body = match serde_json::to_vec(&document) {
            Ok(body) => std::sync::Arc::new(body),
            Err(err) => {
                error!("Failed to serialize webhook payload: {:?}", err);
                return Ok(());
            }
        };

        for webhook in webhooks {
            let delivery = WebhookDelivery::pending(
                realm_id,
                webhook.id,
                payload.event.clone(),
                payload.resource_id,
                document.clone(),
                Utc::now(),
            );
            let delivery_id = delivery.id;
            let attempt_count = delivery.attempt_count;

            if let Err(err) = self.deliveries.enqueue(delivery).await {
                error!(
                    webhook_id = %webhook.id,
                    error = ?err,
                    "failed to record a webhook delivery; the event is lost"
                );
                continue;
            }

            let policy = match self
                .deliveries
                .resolve_retry_policy(realm_id, webhook.id)
                .await
            {
                Ok(policy) => policy,
                Err(_) => RetryPolicy::SYSTEM_DEFAULT,
            };

            let job = DeliveryJob {
                delivery_id,
                realm_id,
                webhook_id: webhook.id,
                event: payload.event.clone(),
                endpoint: webhook.endpoint,
                headers: webhook.headers,
                secret: webhook.secret,
                body: body.clone(),
                attempt_count,
                elapsed: std::time::Duration::ZERO,
                policy,
            };

            if let Err(err) = self.delivery_sender.try_send(job) {
                let reason = match err {
                    mpsc::error::TrySendError::Full(_) => "queue is full",
                    mpsc::error::TrySendError::Closed(_) => "dispatcher is not running",
                };
                warn!(
                    webhook_id = %webhook.id,
                    delivery_id = %delivery_id,
                    reason,
                    "deferring the first webhook delivery attempt to the outbox worker"
                );
            }
        }

        Ok(())
    }
}

/// Integration tests for `PostgresWebhookRepository`'s realm isolation on the write paths
/// (FK-015b). These require a running PostgreSQL instance and are skipped during regular
/// `cargo test` runs. Execute them explicitly with:
///
/// ```text
/// cargo test -p ferriskey-core -- --ignored
/// ```
///
/// Environment variables (defaults shown):
/// ```text
/// DATABASE_URL = postgres://ferriskey:ferriskey@localhost:5432/ferriskey
/// ```
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    async fn setup() -> (PostgresWebhookRepository, RealmId, RealmId) {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("webhook_repository_test_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{}""#, schema)))
            .await
            .expect("create test schema");

        let separator = if base_url.contains('?') { '&' } else { '?' };
        let schema_url = format!("{base_url}{separator}options=-c search_path={schema}");
        let schema_pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");
        sqlx::migrate!("./migrations")
            .run(&schema_pool)
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
            .execute(&schema_pool)
            .await
            .expect("insert test realm");
        }

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        (
            PostgresWebhookRepository::new(db),
            RealmId::from(realm_a),
            RealmId::from(realm_b),
        )
    }

    async fn create_test_webhook(
        repo: &PostgresWebhookRepository,
        realm_id: RealmId,
        endpoint: &str,
    ) -> Webhook {
        repo.create_webhook(
            realm_id,
            Some("original-name".to_string()),
            None,
            endpoint.to_string(),
            HashMap::new(),
            vec![WebhookTrigger::UserCreated],
            RetryPolicyOverride::default(),
        )
        .await
        .expect("create webhook")
    }

    /// FK-015b: `tenant-b` must not be able to overwrite a webhook that belongs to `tenant-a`,
    /// and the attempt must leave the victim row exactly as it was.
    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn update_webhook_refuses_a_target_from_another_realm() {
        let (repo, realm_a, realm_b) = setup().await;
        let webhook = create_test_webhook(&repo, realm_a, "https://example.com/original").await;

        let result = repo
            .update_webhook(
                realm_b,
                webhook.id,
                Some("hijacked".to_string()),
                None,
                "https://attacker.example/hook".to_string(),
                Some(HashMap::new()),
                Vec::new(),
                None,
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::WebhookNotFound)),
            "expected WebhookNotFound, got {result:?}"
        );

        let unchanged = repo
            .get_webhook_by_id(webhook.id, realm_a)
            .await
            .expect("get webhook")
            .expect("webhook still exists in its own realm");

        assert_eq!(unchanged.endpoint, "https://example.com/original");
        assert_eq!(unchanged.name, Some("original-name".to_string()));
    }

    /// FK-015b: `tenant-b` must not be able to delete a webhook that belongs to `tenant-a`.
    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn delete_webhook_refuses_a_target_from_another_realm() {
        let (repo, realm_a, realm_b) = setup().await;
        let webhook = create_test_webhook(&repo, realm_a, "https://example.com/original").await;

        let result = repo.delete_webhook(realm_b, webhook.id).await;

        assert!(
            matches!(result, Err(CoreError::WebhookNotFound)),
            "expected WebhookNotFound, got {result:?}"
        );

        let still_there = repo
            .get_webhook_by_id(webhook.id, realm_a)
            .await
            .expect("get webhook");

        assert!(
            still_there.is_some(),
            "webhook must survive a delete attempted from another realm"
        );
    }

    /// Non-regression: without this, a fix that denied every cross-realm write by accident could
    /// look like a success above. `tenant-a` must retain full read/write control of its own
    /// webhook, and a read afterwards must reflect the update.
    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn update_webhook_succeeds_for_the_owning_realm_and_a_read_reflects_it() {
        let (repo, realm_a, _realm_b) = setup().await;
        let webhook = create_test_webhook(&repo, realm_a, "https://example.com/original").await;

        let updated = repo
            .update_webhook(
                realm_a,
                webhook.id,
                Some("renamed".to_string()),
                None,
                "https://example.com/updated".to_string(),
                Some(HashMap::new()),
                vec![WebhookTrigger::UserDeleted],
                None,
            )
            .await
            .expect("update webhook in its own realm");

        assert_eq!(updated.endpoint, "https://example.com/updated");
        assert_eq!(updated.name, Some("renamed".to_string()));

        let read_back = repo
            .get_webhook_by_id(webhook.id, realm_a)
            .await
            .expect("get webhook")
            .expect("webhook exists");

        assert_eq!(read_back.endpoint, "https://example.com/updated");
        assert_eq!(
            read_back
                .subscribers
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>(),
            vec![WebhookTrigger::UserDeleted]
        );
    }
}
