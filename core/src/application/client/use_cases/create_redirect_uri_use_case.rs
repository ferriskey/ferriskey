use tracing::error;
use uuid::Uuid;

use crate::application::client::policies::ClientPolicy;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookNotifierService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;
use crate::{
    application::common::services::DefaultRedirectUriService,
    domain::client::{
        entities::{ClientError, redirect_uri::RedirectUri},
        ports::RedirectUriService,
        value_objects::CreateRedirectUriRequest,
    },
};

#[derive(Clone)]
pub struct CreateRedirectUriUseCase {
    redirect_uri_service: DefaultRedirectUriService,
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct CreateRedirectUriUseCaseParams {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: CreateRedirectUriRequest,
}

impl CreateRedirectUriUseCase {
    pub fn new(
        redirect_uri_service: DefaultRedirectUriService,
        realm_service: DefaultRealmService,
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
        params: CreateRedirectUriUseCaseParams,
    ) -> Result<RedirectUri, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let can_create_redirect_uri = ClientPolicy::create(
            identity,
            realm,
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await
        .map_err(|_| {
            ClientError::Forbidden("Insufficient permissions to create redirect URI".to_string())
        })?;

        if !can_create_redirect_uri {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to create redirect URI".to_string(),
            ));
        }

        let redirect_uri = self
            .redirect_uri_service
            .add_redirect_uri(params.payload, params.realm_name, params.client_id)
            .await?;

        self.webhook_notifier_service
            .notify(
                redirect_uri.id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriCreated,
                    redirect_uri.id,
                    Some(redirect_uri.clone()),
                ),
            )
            .await
            .map_err(|e| {
                error!("Failed to notify webhook: {}", e);
                ClientError::InternalServerError
            })?;

        Ok(redirect_uri)
    }
}
