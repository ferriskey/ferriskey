use crate::{
    application::common::services::{
        DefaultClientService, DefaultGrantTypeService, DefaultRealmService,
    },
    domain::{
        authentication::{
            entities::{AuthenticationError, GrantType, JwtToken},
            ports::GrantTypeService,
            value_objects::GrantTypeParams,
        },
        client::ports::ClientService,
        realm::ports::RealmService,
    },
};

/// Input for authentication use case
#[derive(Debug, Clone)]
pub struct ExchangeTokenUseCaseParams {
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub grant_type: GrantType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
    pub base_url: String,
}

#[derive(Clone)]
pub struct ExchangeTokenUseCase {
    realm_service: DefaultRealmService,
    client_service: DefaultClientService,
    grant_type_service: DefaultGrantTypeService,
}

impl ExchangeTokenUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        client_service: DefaultClientService,
        grant_type_service: DefaultGrantTypeService,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            grant_type_service,
        }
    }

    pub async fn execute(
        &self,
        input: ExchangeTokenUseCaseParams,
    ) -> Result<JwtToken, AuthenticationError> {
        // 1. Orchestration: Get realm (cross-domain coordination)
        let realm = self
            .realm_service
            .get_by_name(input.realm_name.clone())
            .await
            .map_err(|_| AuthenticationError::InvalidRealm)?;

        // 2. Orchestration: Validate client (cross-domain coordination)
        let _client = self
            .client_service
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidClient)?;

        // 3. Create domain parameters
        let params = GrantTypeParams {
            realm_id: realm.id,
            base_url: input.base_url,
            realm_name: realm.name,
            client_id: input.client_id,
            client_secret: input.client_secret,
            code: input.code,
            username: input.username,
            password: input.password,
            refresh_token: input.refresh_token,
            redirect_uri: None,
        };

        // 4. Delegate to domain service (pure business logic)
        self.grant_type_service
            .authenticate_with_grant_type(input.grant_type, params)
            .await
    }
}
