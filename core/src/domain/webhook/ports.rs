use anyhow::Error;
use uuid::Uuid;

use crate::domain::webhook::entities::{Webhook, WebhookError};

pub trait WebhookService: Clone + Send + Sync {
    fn fetch_webhook_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Webhook>, Error>> + Send;

    fn find_by_id(&self, id: &str) -> impl Future<Output = Result<Option<Webhook>, Error>> + Send;

    fn create_webhook(&self, webhook: Webhook) -> impl Future<Output = Result<(), Error>> + Send;

    fn update_webhook(
        &self,
        id: &str,
        webhook: Webhook,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn delete_webhook(&self, id: &str) -> impl Future<Output = Result<(), Error>> + Send;
}

pub trait WebhookRepository: Clone + Send + Sync + 'static {
    fn fetch_webhook_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    fn get_by_id(
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
        id: &str,
        webhook: Webhook,
    ) -> impl Future<Output = Result<(), WebhookError>> + Send;

    fn delete_webhook(&self, id: &str) -> impl Future<Output = Result<(), WebhookError>> + Send;
}
