use uuid::Uuid;

use crate::{
    domain::webhook::{
        entities::{Webhook, WebhookError},
        ports::WebhookRepository,
    },
    infrastructure::repositories::webhook_repository::PostgresWebhookRepository,
};

#[derive(Clone)]
pub enum WebhookRepoAny {
    Postgres(PostgresWebhookRepository),
}

impl WebhookRepository for WebhookRepoAny {
    async fn fetch_webhook_by_realm(&self, realm_id: Uuid) -> Result<Vec<Webhook>, WebhookError> {
        todo!()
    }

    async fn get_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<Webhook>, WebhookError> {
        todo!()
    }

    async fn create_webhook(
        &self,
        realm_id: Uuid,
        endpoint: String,
        subscribers: Vec<String>,
    ) -> Result<Webhook, WebhookError> {
        todo!()
    }

    async fn update_webhook(&self, id: &str, webhook: Webhook) -> Result<(), WebhookError> {
        todo!()
    }

    async fn delete_webhook(&self, id: &str) -> Result<(), WebhookError> {
        todo!()
    }
}
