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
}

impl CreateClientUseCase {
    pub fn new(realm_service: DefaultRealmService, client_service: DefaultClientService) -> Self {
        Self {
            realm_service,
            client_service,
        }
    }

    pub async fn execute(&self, params: CreateClientUseCaseParams) -> Result<Client, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let secret = (!params.public_client).then(generate_random_string);

        let client = self
            .client_service
            .create_client(
                CreateClientRequest {
                    realm_id: realm.id,
                    name: params.name,
                    client_id: params.client_id,
                    secret,
                    enabled: params.enabled,
                    protocol: params.protocol,
                    public_client: params.public_client,
                    service_account_enabled: params.service_account_enabled,
                    client_type: params.client_type,
                },
                params.realm_name,
            )
            .await?;

        Ok(client)
    }
}
