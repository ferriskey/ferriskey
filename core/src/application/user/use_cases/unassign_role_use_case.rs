use uuid::Uuid;

use crate::{
    application::user::policies::user_role_policy::UserRolePolicy,
    domain::{
        authentication::value_objects::Identity,
        client::ports::ClientService,
        realm::ports::RealmService,
        user::{
            entities::UserError,
            ports::{UserRoleService, UserService},
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
pub struct UnassignRoleUseCase<R, U, C, UR>
where
    R: RealmService,
    U: UserService,
    C: ClientService,
    UR: UserRoleService,
{
    pub realm_service: R,
    pub user_service: U,
    pub client_service: C,
    pub user_role_service: UR,
}

impl<R, U, C, UR> UnassignRoleUseCase<R, U, C, UR>
where
    R: RealmService,
    U: UserService,
    C: ClientService,
    UR: UserRoleService,
{
    pub fn new(
        realm_service: R,
        user_service: U,
        client_service: C,
        user_role_service: UR,
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
