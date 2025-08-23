use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::domain::webhook::{
    entities::{errors::WebhookError, webhook_payload::WebhookPayload},
    ports::{WebhookNotifierService, WebhookRepository},
};

#[derive(Clone)]
pub struct WebhookNotifierServiceImpl<W>
where
    W: WebhookRepository,
{
    webhook_repository: W,
    http_client: Client,
}

impl<W> WebhookNotifierServiceImpl<W>
where
    W: WebhookRepository,
{
    pub fn new(webhook_repository: W) -> Self {
        WebhookNotifierServiceImpl {
            webhook_repository,
            http_client: Client::new(),
        }
    }
}

impl<W> WebhookNotifierService for WebhookNotifierServiceImpl<W>
where
    W: WebhookRepository,
{
    async fn notify<T: Send + Sync + Serialize>(
        &self,
        realm_id: Uuid,
        payload: WebhookPayload<T>,
    ) -> Result<(), WebhookError> {
        let repo = self.webhook_repository.clone();
        let client = self.http_client.clone();

        tokio::spawn(async move {
            let webhooks = repo
                .fetch_webhooks_by_subscriber(realm_id, payload.event)
                .await;

            match webhooks {
                Ok(webhooks) => {
                    for webhook in webhooks {
                        let endpoint = webhook.endpoint.clone();
                        let client = client.clone();

                        tokio::spawn(async move {
                            let response = client
                                .post(endpoint)
                                .json(&json!({ "name": "John Doe" }))
                                .send()
                                .await;

                            if let Err(err) = response {
                                error!("Webhook POST failed: {:?}", err);
                            }
                        });
                    }
                }
                Err(err) => {
                    error!("Failed to fetch webhooks: {:?}", err);
                }
            }
        });

        Ok(())
    }
}
