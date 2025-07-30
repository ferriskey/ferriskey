use uuid::Uuid;

use crate::{
    application::user::policies::user_role_policy::UserRolePolicy,
    domain::{
        authentication::value_objects::Identity,
        client::ports::ClientService,
        realm::ports::RealmService,
        role::entities::Role,
        user::{entities::UserError, ports::UserService},
    },
};

pub struct GetUserRolesUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct GetUserRolesUseCase<R, U, C>
where
    R: RealmService,
    U: UserService,
    C: ClientService,
{
    pub realm_service: R,
    pub user_service: U,
    pub client_service: C,
}
impl<R, U, C> GetUserRolesUseCase<R, U, C>
where
    R: RealmService,
    U: UserService,
    C: ClientService,
{
    pub fn new(realm_service: R, user_service: U, client_service: C) -> Self {
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
