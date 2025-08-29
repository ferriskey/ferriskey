use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultCredentialService, DefaultRealmService, DefaultUserService,
    DefaultWebhookNotifierService,
};
use crate::application::user::policies::user_policy::UserPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::credential::ports::CredentialService;
use crate::domain::realm::ports::RealmService;
use crate::domain::user::entities::UserError;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;
use uuid::Uuid;

#[derive(Clone)]
pub struct ResetPasswordUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    credential_service: DefaultCredentialService,
    webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct ResetPasswordUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
    pub value: String,
    pub temporary: bool,
}

impl ResetPasswordUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        credential_service: DefaultCredentialService,
        webhook_notifier_service: DefaultWebhookNotifierService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            credential_service,
            webhook_notifier_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: ResetPasswordUseCaseParams,
    ) -> Result<(), UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        ensure_permissions(
            UserPolicy::store(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to reset password",
        )
        .map_err(|e| UserError::Forbidden(e.to_string()))?;

        self.credential_service
            .reset_password(params.user_id, params.value, params.temporary)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::<Uuid>::new(
                    WebhookTrigger::AuthResetPassword,
                    params.user_id,
                    None,
                ),
            )
            .await
            .map_err(UserError::FailedWebhookNotification)?;

        Ok(())
    }
}
