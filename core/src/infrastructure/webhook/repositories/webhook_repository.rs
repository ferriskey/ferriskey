use reqwest::Client;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError,
    webhook::{
        entities::{
            webhook::Webhook, webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger,
        },
        ports::WebhookRepository,
    },
};

use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, RelationTrait,
};
use tracing::error;

use crate::domain::common::generate_timestamp;
use crate::domain::webhook::entities::webhook_subscriber::WebhookSubscriber;
use crate::entity::webhook_subscribers::{
    ActiveModel as WebhookSubscriberActiveModel, Column as WebhookSubscriberColumn,
    Entity as WebhookSubscriberEntity,
};
use crate::entity::webhooks::{
    ActiveModel as WebhookActiveModel, Column as WebhookColumn, Entity as WebhookEntity,
    Relation as WebhookRelation,
};

use crate::entity::webhook_subscribers::Model as WebhookSubscriberModel;

#[derive(Debug, Clone)]
pub struct PostgresWebhookRepository {
    pub db: DatabaseConnection,
    pub http_client: Client,
}

impl PostgresWebhookRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            http_client: Client::new(),
        }
    }
}

impl WebhookRepository for PostgresWebhookRepository {
    async fn fetch_webhooks_by_realm(&self, realm_id: Uuid) -> Result<Vec<Webhook>, CoreError> {
        let webhooks = WebhookEntity::find()
            .filter(WebhookColumn::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .iter()
            .map(Webhook::from)
            .collect::<Vec<Webhook>>();

        Ok(webhooks)
    }

    async fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, CoreError> {
        let webhooks = WebhookEntity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                WebhookRelation::WebhookSubscribers.def(),
            )
            .filter(WebhookColumn::RealmId.eq(realm_id))
            .filter(WebhookSubscriberColumn::Name.eq(subscriber.to_string()))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch webhooks by subscriber: {}", e);
                CoreError::InternalServerError
            })?
            .into_iter()
            .map(Webhook::from)
            .collect();

        Ok(webhooks)
    }

    async fn get_webhook_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<Webhook>, CoreError> {
        let webhook = WebhookEntity::find()
            .filter(WebhookColumn::RealmId.eq(realm_id))
            .filter(WebhookColumn::Id.eq(webhook_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .map(Webhook::from);

        Ok(webhook)
    }

    async fn create_webhook(
        &self,
        realm_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let subscription_id = Uuid::new_v7(timestamp);

        let mut webhook = WebhookEntity::insert(WebhookActiveModel {
            id: Set(subscription_id),
            endpoint: Set(endpoint),
            name: Set(name),
            description: Set(description),
            realm_id: Set(realm_id),
            triggered_at: Set(None),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        })
        .exec_with_returning(&self.db)
        .await
        .map(Webhook::from)
        .map_err(|e| {
            error!("Failed to create webhook: {}", e);
            CoreError::InternalServerError
        })?;

        let subscribers_model: Vec<WebhookSubscriberModel> =
            WebhookSubscriberEntity::insert_many(subscribers.iter().map(|value| {
                WebhookSubscriberActiveModel {
                    id: Set(Uuid::new_v7(timestamp)),
                    name: Set(value.to_string()),
                    webhook_id: Set(subscription_id),
                }
            }))
            .exec_with_returning_many(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let subscribers: Vec<WebhookSubscriber> = subscribers_model
            .iter()
            .map(|value| value.clone().try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CoreError::InternalServerError)?;

        webhook.subscribers = subscribers;
        Ok(webhook)
    }
    async fn update_webhook(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, CoreError> {
        let mut webhook = WebhookEntity::update(WebhookActiveModel {
            name: Set(name),
            description: Set(description),
            endpoint: Set(endpoint),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        })
        .filter(WebhookColumn::Id.eq(id))
        .exec(&self.db)
        .await
        .map(Webhook::from)
        .map_err(|_| CoreError::InternalServerError)?;

        let _ = WebhookSubscriberEntity::delete_many()
            .filter(WebhookSubscriberColumn::WebhookId.eq(id))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut derived_subscribers = Vec::new();
        for subscriber in subscribers {
            let (_, timestamp) = generate_timestamp();

            let subscription_id = Uuid::new_v7(timestamp);
            let subscriber = WebhookSubscriberActiveModel {
                id: Set(subscription_id),
                name: Set(subscriber.to_string()),
                webhook_id: Set(subscription_id),
            };

            derived_subscribers.push(subscriber);
        }

        let subscribers = WebhookSubscriberEntity::insert_many(derived_subscribers)
            .exec_with_returning_many(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .iter()
            .map(|value| value.clone().try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CoreError::InternalServerError)?;

        webhook.subscribers = subscribers;
        Ok(webhook)
    }

    async fn delete_webhook(&self, id: Uuid) -> Result<(), CoreError> {
        let _ = WebhookEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        realm_id: Uuid,
        payload: WebhookPayload<T>,
    ) -> Result<(), CoreError> {
        let client = self.http_client.clone();
        let webhooks = self
            .fetch_webhooks_by_subscriber(realm_id, payload.event.clone())
            .await;

        tokio::spawn(async move {
            match webhooks {
                Ok(webhooks) => {
                    for webhook in webhooks {
                        let response = client
                            .post(webhook.endpoint)
                            .json(&payload.clone())
                            .send()
                            .await;

                        if let Err(err) = response {
                            error!("Webhook POST failed: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to fetch webhooks: {:?}", err);
                }
            }
        });

        Ok(())
    }
}
