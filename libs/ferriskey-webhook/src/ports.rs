use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::{Realm, RealmId};

use crate::entities::retry_policy::RetryPolicy;
use crate::entities::webhook_delivery::{
    DeliveryFilter, DeliveryOutcome, DeliveryPage, WebhookDelivery, WebhookDeliveryId,
};
use crate::entities::{
    webhook::Webhook, webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger,
};

pub trait WebhookService: Send + Sync {
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

    fn get_webhook_deliveries(
        &self,
        identity: Identity,
        input: GetWebhookDeliveriesInput,
    ) -> impl Future<Output = Result<DeliveryPage, CoreError>> + Send;

    fn get_webhook_delivery(
        &self,
        identity: Identity,
        input: GetWebhookDeliveryInput,
    ) -> impl Future<Output = Result<WebhookDelivery, CoreError>> + Send;

    fn retry_webhook_delivery(
        &self,
        identity: Identity,
        input: RetryWebhookDeliveryInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait WebhookRepository: Send + Sync {
    fn fetch_webhooks_by_realm(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: RealmId,
        subscriber: WebhookTrigger,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn get_webhook_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Option<Webhook>, CoreError>> + Send;

    fn create_webhook(
        &self,
        realm_id: RealmId,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        headers: HashMap<String, String>,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    /// `headers` is optional: `None` leaves the stored headers untouched, so a
    /// caller editing an unrelated field cannot silently drop them. The API
    /// never returns them, which means a client has no way to send them back.
    #[allow(clippy::too_many_arguments)]
    fn update_webhook(
        &self,
        realm_id: RealmId,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        headers: Option<HashMap<String, String>>,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn delete_webhook(
        &self,
        realm_id: RealmId,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        realm_id: RealmId,
        payload: WebhookPayload<T>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait WebhookDeliveryRepository: Send + Sync {
    fn enqueue(
        &self,
        delivery: WebhookDelivery,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn claim_due(
        &self,
        limit: u32,
        lease: Duration,
    ) -> impl Future<Output = Result<Vec<WebhookDelivery>, CoreError>> + Send;

    fn record_outcome(
        &self,
        id: WebhookDeliveryId,
        outcome: DeliveryOutcome,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn reclaim_expired(
        &self,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn list_by_webhook(
        &self,
        realm_id: RealmId,
        webhook_id: Uuid,
        filter: DeliveryFilter,
    ) -> impl Future<Output = Result<DeliveryPage, CoreError>> + Send;

    fn get(
        &self,
        realm_id: RealmId,
        id: WebhookDeliveryId,
    ) -> impl Future<Output = Result<WebhookDelivery, CoreError>> + Send;

    fn requeue(
        &self,
        realm_id: RealmId,
        id: WebhookDeliveryId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn purge_older_than(
        &self,
        cutoff: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn resolve_retry_policy(
        &self,
        realm_id: RealmId,
        webhook_id: Uuid,
    ) -> impl Future<Output = Result<RetryPolicy, CoreError>> + Send;
}

pub trait WebhookPolicy: Send + Sync {
    fn can_create_webhook(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_webhook(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_webhook(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_webhook(
        &self,
        identity: &Identity,
        target_realm: &Realm,
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
    pub headers: HashMap<String, String>,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct UpdateWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    /// Omitted means "keep what is stored"; `Some` replaces the whole set.
    pub headers: Option<HashMap<String, String>>,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct DeleteWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
}

pub struct GetWebhookDeliveriesInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub filter: DeliveryFilter,
}

pub struct GetWebhookDeliveryInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub delivery_id: WebhookDeliveryId,
}

pub struct RetryWebhookDeliveryInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub delivery_id: WebhookDeliveryId,
}
