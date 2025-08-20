use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookService,
};
use crate::application::webhook::policies::WebhookPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::{Webhook, WebhookError};
use crate::domain::webhook::ports::WebhookService;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetWebhookUseCase {
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    realm_service: DefaultRealmService,
    webhook_service: DefaultWebhookService,
}

pub struct GetWebhookUseCaseParams {
    pub realm_name: String,
    pub webhook_id: Uuid,
}

impl GetWebhookUseCase {
    pub fn new(
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        realm_service: DefaultRealmService,
        webhook_service: DefaultWebhookService,
    ) -> Self {
        Self {
            user_service,
            client_service,
            realm_service,
            webhook_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: GetWebhookUseCaseParams,
    ) -> Result<Option<Webhook>, WebhookError> {
        info!("Getting realm webhooks : {}", params.realm_name);

        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| WebhookError::RealmNotFound)?;

        ensure_permissions(
            WebhookPolicy::view(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to get a webhook",
        )
        .map_err(|_| WebhookError::Forbidden)?;

        let webhook = self
            .webhook_service
            .get_by_id(realm.id, params.webhook_id)
            .await?;

        Ok(webhook)
    }
}
