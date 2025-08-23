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
        user::{entities::UserError, ports::UserService},
        webhook::{
            entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
            ports::WebhookNotifierService,
        },
    },
};
use tracing::error;
use uuid::Uuid;

pub struct BulkDeleteUserUseCaseParams {
    pub realm_name: String,
    pub ids: Vec<Uuid>,
}

#[derive(Clone)]
pub struct BulkDeleteUserUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

impl BulkDeleteUserUseCase {
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
        params: BulkDeleteUserUseCaseParams,
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
            "Insufficient permissions to delete users",
        )?;

        let count = self
            .user_service
            .bulk_delete_user(params.ids.clone())
            .await
            .map_err(|_| UserError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::<Vec<Uuid>>::new(
                    WebhookTrigger::UserBulkDeleted,
                    realm.id,
                    Some(params.ids),
                ),
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
        result_has_permission
            .map_err(|_| UserError::Forbidden(error_message.to_string()))?
            .then_some(())
            .ok_or_else(|| UserError::Forbidden(error_message.to_string()))
    }
}
