use crate::application::client::policies::ClientPolicy;
use crate::application::common::services::DefaultUserService;
use crate::domain::authentication::value_objects::Identity;
use crate::{
    application::common::services::{DefaultClientService, DefaultRealmService},
    domain::{
        client::{
            entities::{Client, ClientError},
            ports::ClientService,
            value_objects::CreateClientRequest,
        },
        common::generate_random_string,
        realm::ports::RealmService,
    },
};

#[derive(Clone)]
pub struct CreateClientUseCase {
    pub realm_service: DefaultRealmService,
    pub client_service: DefaultClientService,
    pub user_service: DefaultUserService,
}

pub struct CreateClientUseCaseParams {
    pub realm_name: String,
    pub name: String,
    pub client_id: String,
    pub client_type: String,
    pub service_account_enabled: bool,
    pub public_client: bool,
    pub protocol: String,
    pub enabled: bool,
    pub direct_access_grants_enabled: bool,
}

impl CreateClientUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        client_service: DefaultClientService,
        user_service: DefaultUserService,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            user_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: CreateClientUseCaseParams,
    ) -> Result<Client, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let realm_id = realm.id;
        let can_create_client = ClientPolicy::create(
            identity,
            realm,
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await?;

        if !can_create_client {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to create client".to_string(),
            ));
        }

        let secret = (!params.public_client).then(generate_random_string);

        let client = self
            .client_service
            .create_client(
                CreateClientRequest {
                    realm_id,
                    name: params.name,
                    client_id: params.client_id,
                    secret,
                    enabled: params.enabled,
                    protocol: params.protocol,
                    public_client: params.public_client,
                    service_account_enabled: params.service_account_enabled,
                    direct_access_grants_enabled: params.direct_access_grants_enabled,
                    client_type: params.client_type,
                },
                params.realm_name,
            )
            .await?;

        Ok(client)
    }
}
