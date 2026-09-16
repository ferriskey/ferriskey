use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::time::{MissedTickBehavior, interval};
use tracing::{error, info, warn};

use ferriskey_seawatch::ports::SecurityEventRepository;

use crate::domain::webhook::ports::{WebhookDeliveryRepository, WebhookRepository};

use super::delivery::{DeliveryJob, deliver_once};

const TICK_INTERVAL: Duration = Duration::from_secs(5);
const CLAIM_BATCH: u32 = 8;
const LEASE: Duration = Duration::from_secs(600);
const SWEEP_INTERVAL: Duration = Duration::from_secs(60 * 60);
const RETENTION_DAYS: i64 = 30;

pub async fn webhook_delivery_worker_task<D, W, S>(deliveries: D, webhooks: W, security_events: S)
where
    D: WebhookDeliveryRepository,
    W: WebhookRepository,
    S: SecurityEventRepository,
{
    let mut ticker = interval(TICK_INTERVAL);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;

        match deliveries.reclaim_expired(Utc::now()).await {
            Ok(0) => {}
            Ok(count) => info!(count, "Reclaimed webhook deliveries with an expired lease"),
            Err(error) => {
                error!(error = ?error, "Failed to reclaim expired webhook delivery leases")
            }
        }

        let claimed = match deliveries.claim_due(CLAIM_BATCH, LEASE).await {
            Ok(claimed) => claimed,
            Err(error) => {
                error!(error = ?error, "Failed to claim due webhook deliveries");
                continue;
            }
        };

        for delivery in claimed {
            let webhook = match webhooks
                .get_webhook_by_id(delivery.webhook_id, delivery.realm_id)
                .await
            {
                Ok(Some(webhook)) => webhook,
                Ok(None) => {
                    warn!(
                        webhook_id = %delivery.webhook_id,
                        delivery_id = %delivery.id,
                        "skipping a delivery whose webhook no longer exists"
                    );
                    continue;
                }
                Err(error) => {
                    error!(
                        webhook_id = %delivery.webhook_id,
                        error = ?error,
                        "Failed to reload the webhook for a claimed delivery"
                    );
                    continue;
                }
            };

            let body = match serde_json::to_vec(&delivery.payload) {
                Ok(body) => body,
                Err(error) => {
                    error!(
                        delivery_id = %delivery.id,
                        error = %error,
                        "Failed to serialize a stored webhook payload"
                    );
                    continue;
                }
            };

            let policy = match deliveries
                .resolve_retry_policy(delivery.realm_id, delivery.webhook_id)
                .await
            {
                Ok(policy) => policy,
                Err(error) => {
                    error!(
                        webhook_id = %delivery.webhook_id,
                        error = ?error,
                        "Failed to resolve the retry policy for a claimed delivery"
                    );
                    continue;
                }
            };

            let elapsed = delivery.elapsed_since_created(Utc::now());

            let job = DeliveryJob {
                delivery_id: delivery.id,
                realm_id: delivery.realm_id,
                webhook_id: delivery.webhook_id,
                event: delivery.event,
                endpoint: webhook.endpoint,
                headers: webhook.headers,
                secret: webhook.secret,
                body: Arc::new(body),
                attempt_count: delivery.attempt_count,
                elapsed,
                policy,
            };

            deliver_once(job, &deliveries, &security_events).await;
        }
    }
}

pub async fn webhook_delivery_retention_task<D>(deliveries: D)
where
    D: WebhookDeliveryRepository,
{
    let mut ticker = interval(SWEEP_INTERVAL);

    loop {
        ticker.tick().await;

        let cutoff = Utc::now() - chrono::Duration::days(RETENTION_DAYS);

        match deliveries.purge_older_than(cutoff).await {
            Ok(0) => {}
            Ok(count) => info!(
                count,
                retention_days = RETENTION_DAYS,
                "Purged webhook deliveries past the retention window"
            ),
            Err(error) => error!(error = ?error, "Failed to purge old webhook deliveries"),
        }
    }
}
