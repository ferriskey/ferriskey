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
            value_objects::UpdateUserRequest,
        },
        webhook::{
            entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
            ports::WebhookNotifierService,
        },
    },
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UpdateUserUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub enabled: bool,
    pub required_actions: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct UpdateUserUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

impl UpdateUserUseCase {
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
        params: UpdateUserUseCaseParams,
    ) -> Result<User, UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Self::ensure_permissions(
            UserPolicy::update(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to update user",
        )?;

        let user = self
            .user_service
            .update_user(
                params.user_id,
                UpdateUserRequest {
                    firstname: params.firstname,
                    lastname: params.lastname,
                    email: params.email,
                    email_verified: params.email_verified.unwrap_or(false),
                    enabled: params.enabled,
                    required_actions: params.required_actions,
                },
            )
            .await?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::new(WebhookTrigger::UserUpdated, user.id, Some(user.clone())),
            )
            .await
            .map_err(UserError::FailedWebhookNotification)?;

        Ok(user)
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
