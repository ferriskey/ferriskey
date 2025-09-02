use uuid::Uuid;

use crate::domain::webhook::{
    entities::{errors::WebhookError, webhook::Webhook, webhook_trigger::WebhookTrigger},
    ports::WebhookRepository,
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
#[derive(Clone)]
pub enum WebhookRepoAny {
    Postgres(PostgresWebhookRepository),
}

impl WebhookRepository for WebhookRepoAny {
    async fn fetch_webhooks_by_realm(&self, realm_id: Uuid) -> Result<Vec<Webhook>, WebhookError> {
        match self {
            Self::Postgres(r) => r.fetch_webhooks_by_realm(realm_id).await,
        }
    }

    async fn get_webhook_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<Webhook>, WebhookError> {
        match self {
            Self::Postgres(r) => r.get_webhook_by_id(webhook_id, realm_id).await,
        }
    }

    async fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, WebhookError> {
        match self {
            Self::Postgres(r) => r.fetch_webhooks_by_subscriber(realm_id, subscriber).await,
        }
    }

    async fn create_webhook(
        &self,
        realm_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        match self {
            Self::Postgres(r) => {
                r.create_webhook(realm_id, name, description, endpoint, subscribers)
                    .await
            }
        }
    }

    async fn update_webhook(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        match self {
            Self::Postgres(r) => {
                r.update_webhook(id, name, description, endpoint, subscribers)
                    .await
            }
        }
    }

    async fn delete_webhook(&self, id: Uuid) -> Result<(), WebhookError> {
        match self {
            Self::Postgres(r) => r.delete_webhook(id).await,
        }
    }
}

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
    async fn fetch_webhooks_by_realm(&self, realm_id: Uuid) -> Result<Vec<Webhook>, WebhookError> {
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

    async fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, WebhookError> {
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
                WebhookError::InternalServerError
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
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
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
            WebhookError::InternalServerError
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
            .map_err(|_| WebhookError::InternalServerError)?;

        let subscribers: Vec<WebhookSubscriber> = subscribers_model
            .iter()
            .map(|value| value.clone().try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| WebhookError::InternalServerError)?;

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
    ) -> Result<Webhook, WebhookError> {
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
        .map_err(|_| WebhookError::InternalServerError)?;

        let _ = WebhookSubscriberEntity::delete_many()
            .filter(WebhookSubscriberColumn::WebhookId.eq(id))
            .exec(&self.db)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

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
            .map_err(|_| WebhookError::InternalServerError)?
            .iter()
            .map(|value| value.clone().try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| WebhookError::InternalServerError)?;

        webhook.subscribers = subscribers;
        Ok(webhook)
    }

    async fn delete_webhook(&self, id: Uuid) -> Result<(), WebhookError> {
        let _ = WebhookEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        Ok(())
    }
}
