use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

use crate::domain::webhook::{
    entities::errors::WebhookError,
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
    async fn notify(&self, realm_id: Uuid, identifier: String) -> Result<(), WebhookError> {
        let available_webhooks = self
            .webhook_repository
            .fetch_webhooks_by_subscriber(realm_id, identifier)
            .await?;

        for webhook in available_webhooks {
            let res = self
                .http_client
                .post(webhook.endpoint)
                .json(&json!({
                    "name": "John Doe",
                }))
                .send()
                .await
                .map_err(|_| WebhookError::InternalServerError);
        }

        Ok(())
    }
}
