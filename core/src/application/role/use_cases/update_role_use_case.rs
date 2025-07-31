use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultRoleService, DefaultUserService,
};
use crate::application::role::policies::RolePolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::role::entities::{Role, RoleError};
use crate::domain::role::ports::RoleService;
use crate::domain::role::value_objects::UpdateRoleRequest;
use uuid::Uuid;

#[derive(Clone)]
pub struct UpdateRoleUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    role_service: DefaultRoleService,
}

pub struct UpdateRoleUseCaseParams {
    pub realm_name: String,
    pub role_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl UpdateRoleUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        role_service: DefaultRoleService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            role_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: UpdateRoleUseCaseParams,
    ) -> Result<Role, RoleError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        Self::ensure_permissions(
            RolePolicy::update(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to update roles in the realm",
        )?;

        self.role_service
            .update_by_id(
                params.role_id,
                UpdateRoleRequest {
                    name: params.name,
                    description: params.description,
                },
            )
            .await
    }

    #[inline]
    fn ensure_permissions(
        result_has_permission: Result<bool, RoleError>,
        error_message: &str,
    ) -> Result<(), RoleError> {
        result_has_permission
            .map_err(|_| RoleError::Forbidden(error_message.to_string()))?
            .then_some(())
            .ok_or_else(|| RoleError::Forbidden(error_message.to_string()))
    }
}
