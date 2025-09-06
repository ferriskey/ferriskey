use crate::application::client::policies::ClientPolicyImpl;
use crate::application::common::services::{
    DefaultClientService, DefaultRedirectUriService, DefaultUserService,
    DefaultWebhookNotifierService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::ClientError;
use crate::domain::client::ports::RedirectUriService;
use crate::domain::realm::ports::RealmRepository;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;
use crate::infrastructure::realm::repositories::RealmRepoAny;
use uuid::Uuid;

#[derive(Clone)]
pub struct DeleteRedirectUriUseCase {
    redirect_uri_service: DefaultRedirectUriService,
    realm_service: RealmRepoAny,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct DeleteRedirectUriUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

impl DeleteRedirectUriUseCase {
    pub fn new(
        redirect_uri_service: DefaultRedirectUriService,
        realm_service: RealmRepoAny,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        webhook_notifier_service: DefaultWebhookNotifierService,
    ) -> Self {
        Self {
            redirect_uri_service,
            realm_service,
            user_service,
            client_service,
            webhook_notifier_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: DeleteRedirectUriUseCaseParams,
    ) -> Result<(), ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let realm_id = realm.id;
        ClientPolicyImpl::delete(
            identity,
            realm,
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await
        .map_err(|_| {
            ClientError::Forbidden("Insufficient permissions to delete redirect URI".to_string())
        })?
        .then_some(())
        .ok_or_else(|| {
            ClientError::Forbidden("Insufficient permissions to delete redirect URI".to_string())
        })?;

        self.redirect_uri_service
            .delete(params.uri_id)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm_id,
                WebhookPayload::<Uuid>::new(
                    WebhookTrigger::RedirectUriUpdated,
                    params.uri_id,
                    None,
                ),
            )
            .await
            .map_err(ClientError::FailedWebhookNotification)?;

        Ok(())
    }
}
