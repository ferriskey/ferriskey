use uuid::Uuid;

use crate::{
    application::common::FerriskeyService,
    domain::{
        authentication::{
            entities::{
                AuthInput, AuthOutput, AuthSession, AuthSessionParams, AuthenticateInput,
                AuthenticateOutput, AuthenticationMethod, AuthorizeRequestInput,
                AuthorizeRequestOutput, CredentialsAuthParams,
            },
            ports::{AuthService, AuthSessionRepository, AuthenticatePort, GrantTypeService},
            value_objects::{GrantTypeParams, Identity},
        },
        client::ports::{ClientRepository, RedirectUriRepository},
        common::entities::app_errors::CoreError,
        jwt::{
            entities::{ClaimsTyp, JwkKey},
            ports::KeyStoreRepository,
        },
        realm::ports::RealmRepository,
        user::ports::UserRepository,
    },
};

pub mod services;

impl AuthService for FerriskeyService {
    async fn auth(&self, input: AuthInput) -> Result<AuthOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        let redirect_uri = input.redirect_uri.clone();

        let client_redirect_uris = self
            .redirect_uri_repository
            .get_enabled_by_client_id(client.id)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?;

        if !client_redirect_uris.iter().any(|uri| {
            if uri.value == redirect_uri {
                return true;
            }

            if let Ok(regex) = regex::Regex::new(&uri.value) {
                return regex.is_match(&redirect_uri);
            }

            false
        }) {
            return Err(CoreError::InvalidClient);
        }

        if !client.enabled {
            return Err(CoreError::InvalidClient);
        }

        let params = AuthSessionParams {
            realm_id: realm.id,
            client_id: client.id,
            redirect_uri,
            response_type: input.response_type,
            scope: input.scope.unwrap_or_default(),
            state: input.state.clone(),
            nonce: None,
            user_id: None,
            code: None,
            authenticated: false,
        };
        let session = self
            .auth_session_repository
            .create(&AuthSession::new(params))
            .await
            .map_err(|_| CoreError::SessionCreateError)?;

        let login_url = format!(
            "?client_id={}&redirect_uri={}&state={}",
            client.client_id,
            input.redirect_uri,
            input.state.unwrap_or_default()
        );

        Ok(AuthOutput { login_url, session })
    }

    async fn get_certs(&self, realm_name: String) -> Result<Vec<JwkKey>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let jwk_keypair = self
            .keystore_repository
            .get_or_generate_key(realm.id)
            .await
            .map_err(|_| CoreError::RealmKeyNotFound)?;

        let jwk_key = jwk_keypair
            .to_jwk_key()
            .map_err(|e| CoreError::InvalidKey(e.to_string()))?;

        Ok(vec![jwk_key])
    }

    async fn exchange_token(
        &self,
        input: crate::domain::authentication::entities::ExchangeTokenInput,
    ) -> Result<crate::domain::authentication::entities::JwtToken, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        self.client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

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

        self.grant_type_strategies
            .authenticate_with_grant_type(input.grant_type, params)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn authorize_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> Result<AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Bearer {
            return Err(CoreError::InternalServerError);
        }

        let user = self
            .user_repository
            .get_by_id(input.claims.sub)
            .await
            .map_err(|e| {
                tracing::error!("faield to get user by id {}: {:?}", input.claims.sub, e);

                CoreError::InvalidUser
            })?;

        self.grant_type_strategies
            .verify_token(input.token, user.realm_id)
            .await?;

        let identity: Identity = match input.claims.is_service_account() {
            true => {
                let client_id = input.claims.client_id.ok_or(CoreError::InvalidClient)?;
                let client_id = Uuid::parse_str(&client_id).map_err(|e| {
                    tracing::error!("failed to parse client id: {:?}", e);
                    CoreError::InvalidClient
                })?;

                let client = self
                    .client_repository
                    .get_by_id(client_id)
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to get client by id {}: {:?}", client_id, e);
                        CoreError::InvalidClient
                    })?;

                Identity::Client(client)
            }
            false => Identity::User(user),
        };

        Ok(AuthorizeRequestOutput { identity })
    }

    async fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> Result<AuthenticateOutput, CoreError> {
        let auth_session = self
            .auth_session_repository
            .get_by_session_code(input.session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        match input.auth_method {
            AuthenticationMethod::ExistingToken { token } => {
                self.authenticate_factory
                    .handle_token_refresh(token, realm.id, auth_session, input.session_code)
                    .await
            }
            AuthenticationMethod::UserCredentials { username, password } => {
                let params = CredentialsAuthParams {
                    realm_name: input.realm_name,
                    client_id: input.client_id,
                    session_code: input.session_code,
                    base_url: input.base_url,
                    username,
                    password,
                };

                self.authenticate_factory
                    .handle_user_credentials_authentication(params, auth_session)
                    .await
            }
        }
    }
}
