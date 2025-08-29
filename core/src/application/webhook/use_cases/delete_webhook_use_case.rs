use tracing::{error, info};
use uuid::Uuid;

use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookNotifierService,
    DefaultWebhookService,
};
use crate::application::webhook::policies::WebhookPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::errors::WebhookError;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::{WebhookNotifierService, WebhookService};

#[derive(Clone)]
pub struct DeleteWebhookUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_service: DefaultWebhookService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct DeleteWebhookUseCaseParams {
    pub realm_name: String,
    pub webhook_id: Uuid,
}

impl DeleteWebhookUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        webhook_service: DefaultWebhookService,
        webhook_notifier_service: DefaultWebhookNotifierService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            webhook_service,
            webhook_notifier_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: DeleteWebhookUseCaseParams,
    ) -> Result<(), WebhookError> {
        info!("Getting realm webhooks : {}", params.realm_name);

        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| WebhookError::RealmNotFound)?;

        ensure_permissions(
            WebhookPolicy::delete(
                identity.clone(),
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to delete a webhook",
        )
        .map_err(|_| WebhookError::Forbidden)?;

        self.webhook_service.delete(params.webhook_id).await?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::<Uuid>::new(WebhookTrigger::WebhookDeleted, realm.id, None),
            )
            .await
            .map_err(|e| {
                error!("Failed to notify webhook: {}", e);
                WebhookError::InternalServerError
            })?;

        Ok(())
    }
}
