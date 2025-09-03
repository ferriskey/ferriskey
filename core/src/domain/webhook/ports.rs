use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    webhook::entities::{
        errors::WebhookError, webhook::Webhook, webhook_payload::WebhookPayload,
        webhook_trigger::WebhookTrigger,
    },
};

pub trait WebhookService: Clone + Send + Sync {
    fn get_webhooks_by_realm(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn get_webhooks_by_subscribers(
        &self,
        identity: Identity,
        realm_name: String,
        subscriber: WebhookTrigger,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn get_webhook(
        &self,
        identity: Identity,
        webhook_id: Uuid,
    ) -> impl Future<Output = Result<Option<Webhook>, CoreError>> + Send;

    fn create_webhook(
        &self,
        identity: Identity,
        realm_name: String,
        input: CreateWebhookInput,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn update_webhook(
        &self,
        identity: Identity,
        webhook_id: Uuid,
        input: UpdateWebhookInput,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn delete_webhook(
        &self,
        identity: Identity,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    #[deprecated]
    fn fetch_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    #[deprecated]
    fn get_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<Webhook>, WebhookError>> + Send;

    #[deprecated]
    fn fetch_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    #[deprecated]
    fn create(
        &self,
        realm_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    #[deprecated]
    fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    #[deprecated]
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
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn update_webhook(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
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

pub struct CreateWebhookInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct UpdateWebhookInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub subscribers: Vec<WebhookTrigger>,
}
