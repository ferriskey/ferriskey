use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookNotifierService,
};
use crate::application::realm::policies::RealmPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::{Realm, RealmError};
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;

#[derive(Clone)]
pub struct UpdateRealmUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct UpdateRealmUseCaseParams {
    pub realm_name: String,
    pub new_realm_name: String,
}

impl UpdateRealmUseCase {
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
        params: UpdateRealmUseCaseParams,
    ) -> Result<Realm, RealmError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| RealmError::Invalid)?;

        let realm_name = realm.name.clone();
        ensure_permissions(
            RealmPolicy::update(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to update realm",
        )
        .map_err(|_| RealmError::Forbidden)?;

        let realm = self
            .realm_service
            .update_realm(realm_name, params.new_realm_name)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::new(WebhookTrigger::RealmUpdated, realm.id, Some(realm.clone())),
            )
            .await
            .map_err(RealmError::FailedWebhookNotification)?;

        Ok(realm)
    }
}
