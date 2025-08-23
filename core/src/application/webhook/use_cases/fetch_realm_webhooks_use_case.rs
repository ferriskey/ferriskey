use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookService,
};
use crate::application::webhook::policies::WebhookPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::{errors::WebhookError, webhook::Webhook};
use crate::domain::webhook::ports::WebhookService;
use tracing::info;

#[derive(Clone)]
pub struct FetchRealmWebhooksUseCase {
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    realm_service: DefaultRealmService,
    webhook_service: DefaultWebhookService,
}

pub struct FetchRealmWebhooksUseCaseParams {
    pub realm_name: String,
}

impl FetchRealmWebhooksUseCase {
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
        params: FetchRealmWebhooksUseCaseParams,
    ) -> Result<Vec<Webhook>, WebhookError> {
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
            "Insufficient permissions to fetch webhooks",
        )
        .map_err(|_| WebhookError::Forbidden)?;

        let webhooks = self.webhook_service.fetch_by_realm(realm.id).await?;
        Ok(webhooks)
    }
}
