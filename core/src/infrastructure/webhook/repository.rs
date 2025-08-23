use uuid::Uuid;

use crate::{
    domain::webhook::{
        entities::{errors::WebhookError, webhook::Webhook, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
    infrastructure::repositories::webhook_repository::PostgresWebhookRepository,
};

#[derive(Clone)]
pub enum WebhookRepoAny {
    Postgres(PostgresWebhookRepository),
}

impl WebhookRepository for WebhookRepoAny {
    async fn fetch_webhooks_by_realm(&self, realm_id: Uuid) -> Result<Vec<Webhook>, WebhookError> {
        match self {
            Self::Postgres(r) => r.fetch_webhooks_by_realm(realm_id).await,
        }
    }

    async fn get_webhook_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<Webhook>, WebhookError> {
        match self {
            Self::Postgres(r) => r.get_webhook_by_id(webhook_id, realm_id).await,
        }
    }

    async fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, WebhookError> {
        match self {
            Self::Postgres(r) => r.fetch_webhooks_by_subscriber(realm_id, subscriber).await,
        }
    }

    async fn create_webhook(
        &self,
        realm_id: Uuid,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        match self {
            Self::Postgres(r) => r.create_webhook(realm_id, endpoint, subscribers).await,
        }
    }

    async fn update_webhook(
        &self,
        id: Uuid,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        match self {
            Self::Postgres(r) => r.update_webhook(id, endpoint, subscribers).await,
        }
    }

    async fn delete_webhook(&self, id: Uuid) -> Result<(), WebhookError> {
        match self {
            Self::Postgres(r) => r.delete_webhook(id).await,
        }
    }
}
