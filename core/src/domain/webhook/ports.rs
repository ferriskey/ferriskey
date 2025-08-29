use serde::Serialize;
use uuid::Uuid;

use crate::domain::webhook::entities::{
    errors::WebhookError, webhook::Webhook, webhook_payload::WebhookPayload,
    webhook_trigger::WebhookTrigger,
};

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

    fn fetch_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    fn create(
        &self,
        realm_id: Uuid,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn update(
        &self,
        id: Uuid,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), WebhookError>> + Send;
}

pub trait WebhookRepository: Clone + Send + Sync + 'static {
    fn fetch_webhooks_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
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
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn update_webhook(
        &self,
        id: Uuid,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn delete_webhook(&self, id: Uuid) -> impl Future<Output = Result<(), WebhookError>> + Send;
}

pub trait WebhookNotifierService: Clone + Send + Sync {
    fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        realm_id: Uuid,
        payload: WebhookPayload<T>,
    ) -> impl Future<Output = Result<(), WebhookError>> + Send;
}
