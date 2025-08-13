use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::common::generate_timestamp;
use crate::domain::webhook::entities::{Webhook, WebhookError, WebhookSubscriber};
use crate::domain::webhook::ports::WebhookRepository;
use crate::entity::webhook_subscribers::{
    ActiveModel as WebhookSubscriberActiveModel, Entity as WebhookSubscriberEntity,
};
use crate::entity::webhooks::{
    ActiveModel as WebhookActiveModel, Column as WebhookColumn, Entity as WebhookEntity,
};

#[derive(Debug, Clone)]
pub struct PostgresWebhookRepository {
    pub db: DatabaseConnection,
}

impl PostgresWebhookRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl WebhookRepository for PostgresWebhookRepository {
    async fn fetch_webhook_by_realm(&self, realm_id: Uuid) -> Result<Vec<Webhook>, WebhookError> {
        let webhooks = WebhookEntity::find()
            .filter(WebhookColumn::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|_| WebhookError::InternalServerError)?
            .iter()
            .map(Webhook::from)
            .collect::<Vec<Webhook>>();

        Ok(webhooks)
    }

    async fn get_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<Webhook>, WebhookError> {
        let webhook = WebhookEntity::find()
            .filter(WebhookColumn::RealmId.eq(realm_id))
            .filter(WebhookColumn::Id.eq(webhook_id))
            .one(&self.db)
            .await
            .map_err(|_| WebhookError::InternalServerError)?
            .map(Webhook::from);

        Ok(webhook)
    }

    async fn create_webhook(
        &self,
        realm_id: Uuid,
        endpoint: String,
        subscribers: Vec<String>,
    ) -> Result<Webhook, WebhookError> {
        let (_, timestamp) = generate_timestamp();
        let subscription_id = Uuid::new_v7(timestamp);

        let new_webhook = WebhookActiveModel {
            id: Set(subscription_id.clone()),
            endpoint: Set(endpoint),
            realm_id: Set(realm_id),
            triggered_at: Set(None),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        let result_insert = new_webhook.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to create webhook: {:?}", e);
            WebhookError::InternalServerError
        })?;

        let subscribers = subscribers
            .iter()
            .map(|value| WebhookSubscriberActiveModel {
                id: Set(Uuid::new_v7(timestamp)),
                name: Set(value.to_string()),
                webhook_id: Set(subscription_id),
            });

        let subscribers_result_insert = WebhookSubscriberEntity::insert_many(subscribers)
            .exec_with_returning_many(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create webhook: {:?}", e);
                WebhookError::InternalServerError
            })?;

        let mut webhook: Webhook = result_insert.into();
        let subscribers: Vec<WebhookSubscriber> = subscribers_result_insert
            .iter()
            .map(|value| value.clone().into())
            .collect();

        webhook.subscribers = subscribers;
        Ok(webhook)
    }
    async fn update_webhook(&self, id: &str, webhook: Webhook) -> Result<(), WebhookError> {
        todo!()
    }

    async fn delete_webhook(&self, id: &str) -> Result<(), WebhookError> {
        todo!()
    }
}
