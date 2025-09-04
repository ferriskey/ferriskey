use crate::application::client::policies::ClientPolicyImpl;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::{
    entities::{Client, ClientError},
    ports::OldClientService,
};
use crate::domain::realm::ports::RealmService;

#[derive(Clone)]
pub struct GetClientsUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
}

pub struct GetClientsUseCaseParams {
    pub realm_name: String,
}

impl GetClientsUseCase {
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
        params: GetClientsUseCaseParams,
    ) -> Result<Vec<Client>, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let realm_id = realm.id;

        let can_view = ClientPolicyImpl::view(
            identity,
            realm,
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await?;

        if !can_view {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to view clients".to_string(),
            ));
        }

        self.client_service
            .get_by_realm_id(realm_id)
            .await
            .map_err(|_| ClientError::InternalServerError)
    }
}
