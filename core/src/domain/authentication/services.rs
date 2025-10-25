use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{
            AuthInput, AuthOutput, AuthSession, AuthSessionParams, AuthenticationMethod,
            AuthorizeRequestOutput, CredentialsAuthParams, ExchangeTokenInput, JwtToken,
        },
        ports::{AuthService, AuthSessionRepository, AuthenticatePort, GrantTypeService},
        value_objects::{GrantTypeParams, Identity},
    },
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::{
        entities::{ClaimsTyp, JwkKey},
        ports::{KeyStoreRepository, RefreshTokenRepository},
    },
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::WebhookRepository,
};

pub mod authenticate;
pub mod grant_type_service;

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC> AuthService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
{
    async fn auth(&self, input: AuthInput) -> Result<AuthOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await?;

        let redirect_uri = input.redirect_uri.clone();

        let client_redirect_uris = self
            .redirect_uri_repository
            .get_enabled_by_client_id(client.id)
            .await?;

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
            .await?
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

    async fn exchange_token(&self, input: ExchangeTokenInput) -> Result<JwtToken, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await?;

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

        self.authenticate_with_grant_type(input.grant_type, params)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn authorize_request(
        &self,
        input: super::entities::AuthorizeRequestInput,
    ) -> Result<super::entities::AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Bearer {
            return Err(CoreError::InternalServerError);
        }

        let user = self.user_repository.get_by_id(input.claims.sub).await?;

        self.verify_token(input.token, user.realm_id).await?;

        let identity: Identity = match input.claims.is_service_account() {
            true => {
                let client_id = input.claims.client_id.ok_or(CoreError::InvalidClient)?;
                let client_id = Uuid::parse_str(&client_id).map_err(|e| {
                    tracing::error!("failed to parse client id: {:?}", e);
                    CoreError::InvalidClient
                })?;

                let client = self.client_repository.get_by_id(client_id).await?;

                Identity::Client(client)
            }
            false => Identity::User(user),
        };

        Ok(AuthorizeRequestOutput { identity })
    }

    async fn authenticate(
        &self,
        input: super::entities::AuthenticateInput,
    ) -> Result<super::entities::AuthenticateOutput, CoreError> {
        let auth_session = self
            .auth_session_repository
            .get_by_session_code(input.session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        match input.auth_method {
            AuthenticationMethod::ExistingToken { token } => {
                self.handle_token_refresh(token, realm.id, auth_session, input.session_code)
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

                self.handle_user_credentials_authentication(params, auth_session)
                    .await
            }
        }
    }
}
