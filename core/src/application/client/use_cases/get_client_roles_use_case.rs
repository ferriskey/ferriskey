use crate::application::client::policies::ClientPolicyImpl;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultRoleService, DefaultUserService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::ClientError;
use crate::domain::realm::ports::RealmService;
use crate::domain::role::entities::Role;
use crate::domain::role::ports::RoleService;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetClientRolesUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    role_service: DefaultRoleService,
}

pub struct GetClientRolesUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
}

impl GetClientRolesUseCase {
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
        params: GetClientRolesUseCaseParams,
    ) -> Result<Vec<Role>, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let can_view = ClientPolicyImpl::view(
            identity,
            realm,
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await?;

        if !can_view {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to view client roles".to_string(),
            ));
        }

        self.role_service
            .get_by_client_id(params.client_id)
            .await
            .map_err(|_| ClientError::InternalServerError)
    }
}
