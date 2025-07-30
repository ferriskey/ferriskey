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

#[derive(Clone)]
pub struct AssignRoleUseCase<R, UR, U, C>
where
    R: RealmService,
    UR: UserRoleService,
    U: UserService,
    C: ClientService,
{
    pub realm_service: R,
    pub user_role_service: UR,
    pub user_service: U,
    pub client_service: C,
}

#[derive(Debug, Clone)]
pub struct AssignRoleUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

impl<R, UR, U, C> AssignRoleUseCase<R, UR, U, C>
where
    R: RealmService,
    UR: UserRoleService,
    U: UserService,
    C: ClientService,
{
    pub fn new(
        realm_service: R,
        user_role_service: UR,
        user_service: U,
        client_service: C,
    ) -> Self {
        Self {
            realm_service,
            user_role_service,
            user_service,
            client_service,
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

        let realm_name = realm.name.clone();

        Self::ensure_permissions(
            UserRolePolicy::store(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to assign role",
        )?;

        self.user_role_service
            .assign_role(realm_name, params.user_id, params.role_id)
            .await
            .map_err(|_| UserError::InternalServerError)
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
