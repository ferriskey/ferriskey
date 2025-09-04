use crate::application::client::policies::ClientPolicyImpl;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::{Client, ClientError};
use crate::domain::client::ports::OldClientService;
use crate::domain::realm::ports::RealmService;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetClientUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
}

pub struct GetClientUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
}

impl GetClientUseCase {
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
        params: GetClientUseCaseParams,
    ) -> Result<Client, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        Self::ensure_permissions(
            ClientPolicyImpl::view(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to view client",
        )?;

        self.client_service
            .get_by_id(params.client_id)
            .await
            .map_err(|_| ClientError::InternalServerError)
    }

    #[inline]
    fn ensure_permissions(
        result_has_permission: Result<bool, ClientError>,
        error_message: &str,
    ) -> Result<(), ClientError> {
        result_has_permission
            .map_err(|_| ClientError::Forbidden(error_message.to_string()))?
            .then_some(())
            .ok_or_else(|| ClientError::Forbidden(error_message.to_string()))
    }
}
