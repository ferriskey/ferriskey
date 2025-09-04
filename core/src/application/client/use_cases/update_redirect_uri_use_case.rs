use crate::application::client::policies::ClientPolicyImpl;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultRedirectUriService, DefaultUserService,
    DefaultWebhookNotifierService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::ClientError;
use crate::domain::client::entities::redirect_uri::RedirectUri;
use crate::domain::client::ports::RedirectUriService;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;

#[derive(Clone)]
pub struct UpdateRedirectUriUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    redirect_uri_service: DefaultRedirectUriService,
    webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct UpdateRedirectUriUseCaseParams {
    pub realm_name: String,
    pub client_id: uuid::Uuid,
    pub redirect_uri_id: uuid::Uuid,
    pub enabled: bool,
}

impl UpdateRedirectUriUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        redirect_uri_service: DefaultRedirectUriService,
        webhook_notifier_service: DefaultWebhookNotifierService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            redirect_uri_service,
            webhook_notifier_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: UpdateRedirectUriUseCaseParams,
    ) -> Result<RedirectUri, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let can_update = ClientPolicyImpl::update(
            identity,
            realm.clone(),
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await?;

        if !can_update {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to update redirect URI".to_string(),
            ));
        }

        let redirect_uri = self
            .redirect_uri_service
            .update_enabled(params.client_id, params.enabled)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriUpdated,
                    redirect_uri.id,
                    Some(redirect_uri.clone()),
                ),
            )
            .await
            .map_err(ClientError::FailedWebhookNotification)?;

        Ok(redirect_uri)
    }
}
