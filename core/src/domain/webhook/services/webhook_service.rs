use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    webhook::{
        entities::{errors::WebhookError, webhook::Webhook, webhook_trigger::WebhookTrigger},
        ports::{CreateWebhookInput, UpdateWebhookInput, WebhookRepository, WebhookService},
    },
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

    async fn get_webhooks_by_realm(
        &self,
        _identity: Identity,
        _realm_name: String,
    ) -> Result<Vec<Webhook>, CoreError> {
        unimplemented!()
    }

    async fn get_webhooks_by_subscribers(
        &self,
        _identity: Identity,
        _realm_name: String,
        _subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, CoreError> {
        unimplemented!()
    }

    async fn get_webhook(
        &self,
        _identity: Identity,
        _webhook_id: Uuid,
    ) -> Result<Option<Webhook>, CoreError> {
        unimplemented!()
    }

    async fn create_webhook(
        &self,
        _identity: Identity,
        _realm_name: String,
        _input: CreateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        unimplemented!()
    }

    async fn update_webhook(
        &self,
        _identity: Identity,
        _webhook_id: Uuid,
        _input: UpdateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        unimplemented!()
    }

    async fn delete_webhook(
        &self,
        _identity: Identity,
        _webhook_id: Uuid,
    ) -> Result<(), CoreError> {
        unimplemented!()
    }
}
