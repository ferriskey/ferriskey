use uuid::Uuid;

use crate::domain::webhook::entities::{Webhook, WebhookError};

pub trait WebhookService: Clone + Send + Sync {
    fn fetch_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    fn get_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<Webhook>, WebhookError>> + Send;

    fn create(
        &self,
        realm_id: Uuid,
        endpoint: String,
        subscribers: Vec<String>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn update(
        &self,
        id: Uuid,
        endpoint: String,
        subscribers: Vec<String>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), WebhookError>> + Send;
}

pub trait WebhookRepository: Clone + Send + Sync + 'static {
    fn fetch_webhooks_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    fn get_webhook_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<Webhook>, WebhookError>> + Send;

    fn create_webhook(
        &self,
        realm_id: Uuid,
        endpoint: String,
        subscribers: Vec<String>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn update_webhook(
        &self,
        id: Uuid,
        endpoint: String,
        subscribers: Vec<String>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn delete_webhook(&self, id: Uuid) -> impl Future<Output = Result<(), WebhookError>> + Send;
}
