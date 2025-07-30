use uuid::Uuid;

use crate::{
    application::user::policies::user_policy::UserPolicy,
    domain::{
        authentication::value_objects::Identity,
        client::ports::ClientService,
        realm::ports::RealmService,
        user::{entities::UserError, ports::UserService},
    },
};

pub struct BulkDeleteUserUseCaseParams {
    pub realm_name: String,
    pub ids: Vec<Uuid>,
}

#[derive(Clone)]
pub struct BulkDeleteUserUseCase<R, U, C>
where
    R: RealmService,
    U: UserService,
    C: ClientService,
{
    pub realm_service: R,
    pub user_service: U,
    pub client_service: C,
}

impl<R, U, C> BulkDeleteUserUseCase<R, U, C>
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
        params: BulkDeleteUserUseCaseParams,
    ) -> Result<u64, UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Self::ensure_permissions(
            UserPolicy::delete(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to delete users",
        )?;

        let count = self
            .user_service
            .bulk_delete_user(params.ids)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Ok(count)
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
