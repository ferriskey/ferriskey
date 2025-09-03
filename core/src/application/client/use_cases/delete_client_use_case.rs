use uuid::Uuid;

use crate::{
    application::{
        client::policies::ClientPolicy,
        common::services::{
            DefaultClientService, DefaultRealmService, DefaultUserService,
            DefaultWebhookNotifierService,
        },
    },
    domain::{
        authentication::value_objects::Identity,
        client::{entities::ClientError, ports::OldClientService},
        realm::ports::RealmService,
        webhook::{
            entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
            ports::WebhookNotifierService,
        },
    },
};

#[derive(Clone)]
pub struct DeleteClientUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct DeleteClientUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
}

impl DeleteClientUseCase {
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
        params: DeleteClientUseCaseParams,
    ) -> Result<(), ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        Self::ensure_permissions(
            ClientPolicy::delete(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to delete client",
        )?;

        self.client_service.delete_by_id(params.client_id).await?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::<Uuid>::new(WebhookTrigger::ClientDeleted, realm.id, None),
            )
            .await
            .map_err(ClientError::FailedWebhookNotification)?;

        Ok(())
    }

    #[inline]
    fn ensure_permissions(
        result_has_permission: Result<bool, ClientError>,
        error_message: &str,
    ) -> Result<(), ClientError> {
        result_has_permission
            .map_err(|_| ClientError::Forbidden(error_message.to_string()))?
            .then_some(())
            .ok_or_else(|| ClientError::Forbidden(error_message.to_string()))
    }
}
