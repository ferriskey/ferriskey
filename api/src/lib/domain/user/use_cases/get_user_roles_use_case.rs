use uuid::Uuid;

use crate::{
    application::auth::Identity,
    domain::{
        client::services::client_service::DefaultClientService,
        realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
        role::entities::models::Role,
        user::{
            entities::error::UserError, policies::user_role_policy::UserRolePolicy,
            ports::user_service::UserService, services::user_service::DefaultUserService,
        },
    },
};

pub struct GetUserRolesUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct GetUserRolesUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
}

impl GetUserRolesUseCase {
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
        params: GetUserRolesUseCaseParams,
    ) -> Result<Vec<Role>, UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Self::ensure_permissions(
            UserRolePolicy::view(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to view user roles",
        )?;

        self.user_service.get_user_roles(params.user_id).await
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
