use crate::application::client::policies::ClientPolicy;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookNotifierService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::{Client, ClientError};
use crate::domain::client::ports::OldClientService;
use crate::domain::client::value_objects::UpdateClientRequest;
use crate::domain::realm::ports::RealmService;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;
use uuid::Uuid;

#[derive(Clone)]
pub struct UpdateClientUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct UpdateClientUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
    pub payload: UpdateClientRequest,
}

impl UpdateClientUseCase {
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
        params: UpdateClientUseCaseParams,
    ) -> Result<Client, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let realm_name = realm.name.clone();
        let can_update = ClientPolicy::update(
            identity,
            realm.clone(),
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await?;

        if !can_update {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to update client".to_string(),
            ));
        }

        let client = self
            .client_service
            .update_client(params.client_id, realm_name, params.payload)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::new(
                    WebhookTrigger::ClientUpdated,
                    client.id,
                    Some(client.clone()),
                ),
            )
            .await
            .map_err(ClientError::FailedWebhookNotification)?;

        Ok(client)
    }
}
