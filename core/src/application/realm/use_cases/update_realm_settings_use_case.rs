use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookNotifierService,
};
use crate::application::realm::policies::RealmPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::{RealmError, RealmSetting};
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;

#[derive(Clone)]
pub struct UpdateRealmSettingsUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct UpdateRealmSettingsUseCaseParams {
    pub realm_name: String,
    pub algorithm: String,
}

impl UpdateRealmSettingsUseCase {
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
        params: UpdateRealmSettingsUseCaseParams,
    ) -> Result<RealmSetting, RealmError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| RealmError::Invalid)?;

        ensure_permissions(
            RealmPolicy::update(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to update realm settings",
        )
        .map_err(|_| RealmError::Forbidden)?;

        let realm_settings = self
            .realm_service
            .update_realm_setting(realm.id, params.algorithm)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::new(
                    WebhookTrigger::RealmSettingsUpdated,
                    realm.id,
                    Some(realm_settings.clone()),
                ),
            )
            .await
            .map_err(RealmError::FailedWebhookNotification)?;

        Ok(realm_settings)
    }
}
