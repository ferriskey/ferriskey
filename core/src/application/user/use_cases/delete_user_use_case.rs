use tracing::error;
use uuid::Uuid;

use crate::{
    application::{
        common::services::{
            DefaultClientService, DefaultRealmService, DefaultUserService,
            DefaultWebhookNotifierService,
        },
        user::policies::user_policy::UserPolicy,
    },
    domain::{
        authentication::value_objects::Identity,
        realm::ports::RealmService,
        user::{
            entities::{User, UserError},
            ports::UserService,
        },
        webhook::{
            entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
            ports::WebhookNotifierService,
        },
    },
};

#[derive(Debug, Clone)]
pub struct DeleteUserUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct DeleteUserUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

impl DeleteUserUseCase {
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
        params: DeleteUserUseCaseParams,
    ) -> Result<u64, UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Self::ensure_permissions(
            UserPolicy::delete(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to delete a user",
        )?;

        let count = self.user_service.delete_user(params.user_id).await?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::<User>::new(WebhookTrigger::UserDeleted, params.user_id, None),
            )
            .await
            .map_err(|e| {
                error!("Failed to notify webhook: {}", e);
                UserError::InternalServerError
            })?;

        Ok(count)
    }

    #[inline]
    fn ensure_permissions(
        result_has_permission: Result<bool, UserError>,
        error_message: &str,
    ) -> Result<(), UserError> {
        match result_has_permission {
            Ok(true) => Ok(()),
            _ => Err(UserError::Forbidden(error_message.to_string())),
        }
    }
}
