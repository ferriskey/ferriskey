use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultRoleService, DefaultUserService,
};
use crate::application::role::ensure_permissions;
use crate::application::role::policies::RolePolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::role::entities::RoleError;
use crate::domain::role::ports::RoleService;
use uuid::Uuid;

/// Parameters required to delete a role.
pub struct DeleteRoleUseCaseParams {
    pub realm_name: String,
    pub role_id: Uuid,
}

#[derive(Clone)]
pub struct DeleteRoleUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    role_service: DefaultRoleService,
}

impl DeleteRoleUseCase {
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
        params: DeleteRoleUseCaseParams,
    ) -> Result<(), RoleError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        ensure_permissions(
            RolePolicy::delete(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to delete role in the realm",
        )?;

        self.role_service
            .delete_by_id(params.role_id)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        Ok(())
    }
}
