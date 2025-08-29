use serde_json::json;
use uuid::Uuid;

use crate::{
    application::{
        common::services::{
            DefaultClientService, DefaultRealmService, DefaultUserRoleService, DefaultUserService,
            DefaultWebhookNotifierService,
        },
        user::policies::user_role_policy::UserRolePolicy,
    },
    domain::{
        authentication::value_objects::Identity,
        realm::ports::RealmService,
        user::{entities::UserError, ports::UserRoleService},
        webhook::{
            entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
            ports::WebhookNotifierService,
        },
    },
};

#[derive(Clone)]
pub struct AssignRoleUseCase {
    pub realm_service: DefaultRealmService,
    pub user_role_service: DefaultUserRoleService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

#[derive(Debug, Clone)]
pub struct AssignRoleUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

impl AssignRoleUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_role_service: DefaultUserRoleService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        webhook_notifier_service: DefaultWebhookNotifierService,
    ) -> Self {
        Self {
            realm_service,
            user_role_service,
            user_service,
            client_service,
            webhook_notifier_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: AssignRoleUseCaseParams,
    ) -> Result<(), UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Self::ensure_permissions(
            UserRolePolicy::store(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to assign role",
        )?;

        self.user_role_service
            .assign_role(realm.name.clone(), params.user_id, params.role_id)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::new(
                    WebhookTrigger::UserAssignRole,
                    params.user_id,
                    Some(json!({ "role_id": params.role_id })),
                ),
            )
            .await
            .map_err(UserError::FailedWebhookNotification)?;

        Ok(())
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
