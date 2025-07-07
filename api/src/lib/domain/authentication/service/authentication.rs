use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{
            dto::AuthenticateDto, error::AuthenticationError, grant_type::GrantType,
            jwt_token::JwtToken,
        },
        grant_type_strategies::{
            authorization_code_strategy::AuthorizationCodeStrategy,
            client_credentials_strategy::ClientCredentialsStrategy,
            password_strategy::PasswordStrategy, refresh_token_strategy::RefreshTokenStrategy,
        },
        ports::{
            auth_session::AuthSessionService,
            authentication::AuthenticationService,
            grant_type_strategy::{GrantTypeParams, GrantTypeStrategy},
        },
    },
    client::{
        ports::client_service::ClientService, services::client_service::DefaultClientService,
    },
    credential::{
        ports::credential_service::CredentialService,
        services::credential_service::DefaultCredentialService,
    },
    jwt::services::jwt_service::DefaultJwtService,
    realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
    utils::generate_random_string,
};

use super::auth_session::DefaultAuthSessionService;

pub type DefaultAuthenticationService = AuthenticationServiceImpl;

#[derive(Clone)]
pub struct AuthenticationServiceImpl {
    pub realm_service: DefaultRealmService,
    pub client_service: DefaultClientService,
    pub credential_service: DefaultCredentialService,
    pub user_service: DefaultUserService,
    pub jwt_service: DefaultJwtService,
    pub auth_session_service: DefaultAuthSessionService,
    pub client_credentials_strategy: ClientCredentialsStrategy,
    pub refresh_token_strategy: RefreshTokenStrategy,
    pub password_strategy: PasswordStrategy,
    pub authorization_code_strategy: AuthorizationCodeStrategy,
}

impl AuthenticationServiceImpl {
    pub fn new(
        realm_service: DefaultRealmService,
        client_service: DefaultClientService,
        credential_service: DefaultCredentialService,
        user_service: DefaultUserService,
        jwt_service: DefaultJwtService,
        auth_session_service: DefaultAuthSessionService,
    ) -> Self {
        let client_credentials_strategy = ClientCredentialsStrategy::new(
            client_service.clone(),
            user_service.clone(),
            jwt_service.clone(),
        );

        let refresh_token_strategy = RefreshTokenStrategy::new(
            jwt_service.clone(),
            client_service.clone(),
            user_service.clone(),
        );

        let password_strategy = PasswordStrategy::new(
            jwt_service.clone(),
            user_service.clone(),
            credential_service.clone(),
            client_service.clone(),
        );

        let authorization_code_strategy = AuthorizationCodeStrategy::new(
            jwt_service.clone(),
            client_service.clone(),
            user_service.clone(),
            credential_service.clone(),
            auth_session_service.clone(),
        );

        Self {
            realm_service,
            client_service,
            credential_service,
            user_service,
            jwt_service,
            auth_session_service,
            client_credentials_strategy,
            refresh_token_strategy,
            password_strategy,
            authorization_code_strategy,
        }
    }
}

impl AuthenticationService for AuthenticationServiceImpl {
    async fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
    ) -> Result<String, AuthenticationError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| AuthenticationError::InvalidRealm)?;

        let _ = self
            .client_service
            .get_by_client_id(client_id, realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidClient);

        let user = self
            .user_service
            .get_by_username(username, realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidUser)?;

        let has_valid_password = self
            .credential_service
            .verify_password(user.id, password)
            .await
            .map_err(|_| AuthenticationError::InvalidPassword);

        if has_valid_password? {
            self.auth_session_service
                .get_by_session_code(session_code)
                .await
                .map_err(|_| AuthenticationError::InternalServerError)?;

            Ok(generate_random_string())
        } else {
            Err(AuthenticationError::InvalidPassword)
        }
    }

    async fn authenticate(&self, data: AuthenticateDto) -> Result<JwtToken, AuthenticationError> {
        let realm = self
            .realm_service
            .get_by_name(data.realm_name.clone())
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let params = GrantTypeParams {
            realm_id: realm.id,
            realm_name: realm.name,
            client_id: data.client_id.clone(),
            client_secret: data.client_secret.clone(),
            code: data.code.clone(),
            username: data.username.clone(),
            password: data.password.clone(),
            refresh_token: data.refresh_token.clone(),
            redirect_uri: None,
        };

        match data.grant_type {
            GrantType::Code => self.authorization_code_strategy.execute(params).await,
            GrantType::Password => self.password_strategy.execute(params).await,
            GrantType::Credentials => self.client_credentials_strategy.execute(params).await,
            GrantType::RefreshToken => self.refresh_token_strategy.execute(params).await,
        }
    }
}
