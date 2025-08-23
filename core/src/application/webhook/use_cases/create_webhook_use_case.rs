use tracing::info;

use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookService,
};
use crate::application::webhook::policies::WebhookPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::entities::{errors::WebhookError, webhook::Webhook};
use crate::domain::webhook::ports::WebhookService;

#[derive(Clone)]
pub struct CreateWebhookUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_service: DefaultWebhookService,
}

pub struct CreateWebhookUseCaseParams {
    pub realm_name: String,
    pub endpoint: String,
    pub subscribers: Vec<WebhookTrigger>,
}

impl CreateWebhookUseCase {
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
        params: CreateWebhookUseCaseParams,
    ) -> Result<Webhook, WebhookError> {
        info!("Getting realm webhooks : {}", params.realm_name);

        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| WebhookError::RealmNotFound)?;

        ensure_permissions(
            WebhookPolicy::create(
                identity.clone(),
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to create a webhook",
        )
        .map_err(|_| WebhookError::Forbidden)?;

        let webhook = self
            .webhook_service
            .create(realm.id, params.endpoint, params.subscribers)
            .await?;

        Ok(webhook)
    }
}
