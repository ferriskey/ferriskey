use uuid::Uuid;

use crate::{
    application::auth::Identity,
    domain::{
        client::services::client_service::DefaultClientService,
        realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
        user::{
            entities::error::UserError,
            policies::user_role_policy::UserRolePolicy,
            ports::user_role_service::UserRoleService,
            services::{
                user_role_service::DefaultUserRoleService, user_service::DefaultUserService,
            },
        },
    },
};

#[derive(Debug, Clone)]
pub struct UnassignRoleUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Clone)]
pub struct UnassignRoleUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub user_role_service: DefaultUserRoleService,
}

impl UnassignRoleUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        user_role_service: DefaultUserRoleService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            user_role_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: UnassignRoleUseCaseParams,
    ) -> Result<(), UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Self::ensure_permissions(
            UserRolePolicy::delete(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to unassign role",
        )?;

        self.user_role_service
            .revoke_role(params.user_id, params.role_id)
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
