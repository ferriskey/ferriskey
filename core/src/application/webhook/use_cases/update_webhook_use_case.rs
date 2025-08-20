use tracing::info;
use uuid::Uuid;

use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookService,
};
use crate::application::webhook::policies::WebhookPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::{Webhook, WebhookError};
use crate::domain::webhook::ports::WebhookService;

#[derive(Clone)]
pub struct UpdateWebhookUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_service: DefaultWebhookService,
}

pub struct UpdateWebhookUseCaseParams {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub endpoint: String,
    pub subscribers: Vec<String>,
}

impl UpdateWebhookUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        webhook_service: DefaultWebhookService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            webhook_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: UpdateWebhookUseCaseParams,
    ) -> Result<Webhook, WebhookError> {
        info!("Getting realm webhooks : {}", params.realm_name);

        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| WebhookError::RealmNotFound)?;

        ensure_permissions(
            WebhookPolicy::update(
                identity.clone(),
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to update a webhook",
        )
        .map_err(|_| WebhookError::Forbidden)?;

        let webhook = self
            .webhook_service
            .update(params.webhook_id, params.endpoint, params.subscribers)
            .await?;

        Ok(webhook)
    }
}
