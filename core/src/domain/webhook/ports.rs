use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    webhook::entities::{
        errors::WebhookError, webhook::Webhook, webhook_payload::WebhookPayload,
        webhook_trigger::WebhookTrigger,
    },
};

pub trait WebhookService: Clone + Send + Sync {
    fn get_webhooks_by_realm(
        &self,
        identity: Identity,
        input: GetWebhooksInput,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn get_webhooks_by_subscribers(
        &self,
        identity: Identity,
        input: GetWebhookSubscribersInput,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn get_webhook(
        &self,
        identity: Identity,
        input: GetWebhookInput,
    ) -> impl Future<Output = Result<Option<Webhook>, CoreError>> + Send;

    fn create_webhook(
        &self,
        identity: Identity,
        input: CreateWebhookInput,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn update_webhook(
        &self,
        identity: Identity,
        input: UpdateWebhookInput,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn delete_webhook(
        &self,
        identity: Identity,
        input: DeleteWebhookInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
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

pub trait WebhookNotifierRepository: Clone + Send + Sync + 'static {
    fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        webhooks: Vec<Webhook>,
        payload: WebhookPayload<T>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait WebhookNotifierService: Clone + Send + Sync {
    fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        realm_id: Uuid,
        payload: WebhookPayload<T>,
    ) -> impl Future<Output = Result<(), WebhookError>> + Send;
}

pub trait WebhookPolicy: Clone + Send + Sync + 'static {
    fn can_create_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub struct GetWebhooksInput {
    pub realm_name: String,
}

pub struct GetWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
}

pub struct GetWebhookSubscribersInput {
    pub realm_name: String,
    pub subscriber: WebhookTrigger,
}

pub struct CreateWebhookInput {
    pub realm_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct UpdateWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct DeleteWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
}
