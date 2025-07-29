use uuid::Uuid;

use crate::{
    application::auth::Identity,
    domain::{
        client::services::client_service::DefaultClientService,
        realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
        user::{
            dtos::user_dto::UpdateUserDto,
            entities::{error::UserError, model::User},
            policies::user_policy::UserPolicy,
            ports::user_service::UserService,
            services::user_service::DefaultUserService,
        },
    },
};

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
}

impl UpdateUserUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: UpdateUserUseCaseParams,
    ) -> Result<User, UserError> {
        // Implementation goes here
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Self::ensure_permissions(
            UserPolicy::update(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to update user",
        )?;

        self.user_service
            .update_user(
                params.user_id,
                UpdateUserDto {
                    firstname: params.firstname,
                    lastname: params.lastname,
                    email: params.email,
                    email_verified: params.email_verified.unwrap_or(false),
                    enabled: params.enabled,
                    required_actions: params.required_actions,
                },
            )
            .await
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
