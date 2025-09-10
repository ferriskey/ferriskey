use reqwest::Client;
use serde::Serialize;

use crate::domain::{
    common::entities::app_errors::CoreError,
    webhook::{
        entities::{webhook::Webhook, webhook_payload::WebhookPayload},
        ports::WebhookNotifierRepository,
    },
};
use tracing::error;

#[derive(Clone)]
pub enum WebhookNotifierRepoAny {
    Postgres(PostgresWebhookNotifierRepository),
}

impl WebhookNotifierRepository for WebhookNotifierRepoAny {
    async fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        webhooks: Vec<Webhook>,
        payload: WebhookPayload<T>,
    ) -> Result<(), CoreError> {
        match self {
            WebhookNotifierRepoAny::Postgres(r) => r.notify(webhooks, payload).await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresWebhookNotifierRepository {
    pub http_client: Client,
}

impl PostgresWebhookNotifierRepository {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }
}

impl WebhookNotifierRepository for PostgresWebhookNotifierRepository {
    async fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        webhooks: Vec<Webhook>,
        payload: WebhookPayload<T>,
    ) -> Result<(), CoreError> {
        let client = self.http_client.clone();

        tokio::spawn(async move {
            for webhook in webhooks {
                let response = client
                    .clone()
                    .post(webhook.endpoint)
                    .json(&payload.clone())
                    .send()
                    .await;

                if let Err(err) = response {
                    error!("Webhook POST failed: {:?}", err);
                }
            }
        });

        Ok(())
    }
}
