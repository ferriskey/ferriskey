use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookNotifierService,
};
use crate::application::realm::policies::RealmPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::RealmError;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct DeleteRealmUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct DeleteRealmUseCaseParams {
    pub realm_name: String,
}

impl DeleteRealmUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        webhook_notifier_service: DefaultWebhookNotifierService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            webhook_notifier_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: DeleteRealmUseCaseParams,
    ) -> Result<(), RealmError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| RealmError::Invalid)?;

        let realm_name = realm.name.clone();

        if realm_name == "master" {
            return Err(RealmError::CannotDeleteMaster);
        }

        ensure_permissions(
            RealmPolicy::delete(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to delete realm",
        )
        .map_err(|_| RealmError::Forbidden)?;

        self.realm_service
            .delete_by_name(realm_name)
            .await
            .map_err(|_| RealmError::Forbidden)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::<Uuid>::new(WebhookTrigger::RealmDeleted, realm.id, None),
            )
            .await
            .map_err(|e| {
                error!("Failed to notify webhook: {}", e);
                RealmError::InternalServerError
            })?;

        Ok(())
    }
}
