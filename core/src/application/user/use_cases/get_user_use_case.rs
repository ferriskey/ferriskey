use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService,
};
use crate::application::user::policies::user_policy::UserPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::ports::RealmService;
use crate::domain::user::entities::{User, UserError};
use crate::domain::user::ports::UserService;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetUserUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
}

pub struct GetUserUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
}

impl GetUserUseCase {
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
        params: GetUserUseCaseParams,
    ) -> Result<User, UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        ensure_permissions(
            UserPolicy::view(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to view user",
        )
        .map_err(|e| UserError::Forbidden(e.to_string()))?;

        self.user_service
            .get_by_id(params.user_id)
            .await
            .map_err(|_| UserError::InternalServerError)
    }
}
