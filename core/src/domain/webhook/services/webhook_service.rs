use uuid::Uuid;

use crate::domain::webhook::{
    entities::{errors::WebhookError, webhook::Webhook, webhook_trigger::WebhookTrigger},
    ports::{WebhookRepository, WebhookService},
};

#[derive(Clone)]
pub struct WebhookServiceImpl<W>
where
    W: WebhookRepository,
{
    webhook_repository: W,
}

impl<W> WebhookServiceImpl<W>
where
    W: WebhookRepository,
{
    pub fn new(webhook_repository: W) -> Self {
        Self { webhook_repository }
    }
}

impl<W> WebhookService for WebhookServiceImpl<W>
where
    W: WebhookRepository,
{
    async fn fetch_by_realm(&self, realm_id: Uuid) -> Result<Vec<Webhook>, WebhookError> {
        self.webhook_repository
            .fetch_webhooks_by_realm(realm_id)
            .await
    }

    async fn fetch_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, WebhookError> {
        self.webhook_repository
            .fetch_webhooks_by_subscriber(realm_id, subscriber)
            .await
    }

    async fn get_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<Webhook>, WebhookError> {
        self.webhook_repository
            .get_webhook_by_id(webhook_id, realm_id)
            .await
    }

    async fn create(
        &self,
        realm_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        self.webhook_repository
            .create_webhook(realm_id, name, description, endpoint, subscribers)
            .await
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        self.webhook_repository
            .update_webhook(id, name, description, endpoint, subscribers)
            .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), WebhookError> {
        self.webhook_repository.delete_webhook(id).await
    }
}
