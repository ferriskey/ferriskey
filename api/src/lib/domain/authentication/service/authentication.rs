use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, grant_type::GrantType, jwt_token::JwtToken},
        grant_type_strategies::{
            client_credentials_strategy::ClientCredentialsStrategy,
            refresh_token_strategy::RefreshTokenStrategy,
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
    jwt::{
        entities::jwt_claim::JwtClaim, ports::jwt_service::JwtService,
        services::jwt_service::DefaultJwtService,
    },
    realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
    utils::generate_random_string,
};

use super::auth_session::DefaultAuthSessionService;

pub type DefaultAuthenticationService = AuthenticationServiceImpl;

#[derive(Clone)]
pub struct AuthenticationServiceImpl {
    pub realm_service: Arc<DefaultRealmService>,
    pub client_service: Arc<DefaultClientService>,
    pub credential_service: Arc<DefaultCredentialService>,
    pub user_service: Arc<DefaultUserService>,
    pub jwt_service: Arc<DefaultJwtService>,
    pub auth_session_service: Arc<DefaultAuthSessionService>,
    pub client_credentials_strategy: ClientCredentialsStrategy,
    pub refresh_token_strategy: RefreshTokenStrategy,
}

impl AuthenticationServiceImpl {
    pub fn new(
        realm_service: Arc<DefaultRealmService>,
        client_service: Arc<DefaultClientService>,
        credential_service: Arc<DefaultCredentialService>,
        user_service: Arc<DefaultUserService>,
        jwt_service: Arc<DefaultJwtService>,
        auth_session_service: Arc<DefaultAuthSessionService>,
    ) -> Self {
        let client_credentials_strategy = ClientCredentialsStrategy::new(
            client_service.clone(),
            user_service.clone(),
            jwt_service.clone(),
        );

        let refresh_token_strategy =
            RefreshTokenStrategy::new(jwt_service.clone(), client_service.clone());

        Self {
            realm_service,
            client_service,
            credential_service,
            user_service,
            jwt_service,
            auth_session_service,
            client_credentials_strategy,
            refresh_token_strategy,
        }
    }
}

impl AuthenticationService for AuthenticationServiceImpl {
    async fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> Result<JwtToken, AuthenticationError> {
        let auth_session = self
            .auth_session_service
            .get_by_code(code)
            .await
            .map_err(|_| AuthenticationError::Invalid)?;

        let user_id = auth_session.user_id.ok_or(AuthenticationError::Invalid)?;

        let user = self
            .user_service
            .get_by_id(user_id)
            .await
            .map_err(|_| AuthenticationError::Invalid)?;

        let claims = JwtClaim::new(
            user.id,
            user.username,
            "http://localhost:3333/realms/master".to_string(),
            vec!["master-realm".to_string(), "account".to_string()],
            "Bearer".to_string(),
            client_id,
        );

        let jwt = self
            .jwt_service
            .generate_token(claims)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let jwt_token = JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            "8xLOxBtZp8".to_string(),
            3600,
            "id_token".to_string(),
        );

        Ok(jwt_token)
    }

    async fn using_password(
        &self,
        realm_id: Uuid,
        client_id: String,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError> {
        let user = self
            .user_service
            .get_by_username(username, realm_id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let credential = self
            .credential_service
            .verify_password(user.id, password)
            .await;

        let is_valid = match credential {
            Ok(is_valid) => is_valid,
            Err(_) => return Err(AuthenticationError::Invalid),
        };

        if !is_valid {
            return Err(AuthenticationError::Invalid);
        }

        let claims = JwtClaim::new(
            user.id,
            user.username,
            "http://localhost:3333/realms/master".to_string(),
            vec!["master-realm".to_string(), "account".to_string()],
            "Bearer".to_string(),
            client_id,
        );
        let jwt = self
            .jwt_service
            .generate_token(claims)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let jwt_token = JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            "8xLOxBtZp8".to_string(),
            3600,
            "id_token".to_string(),
        );

        Ok(jwt_token)
    }

    async fn authentificate(
        &self,
        realm_name: String,
        grant_type: GrantType,
        client_id: String,
        client_secret: Option<String>,
        code: Option<String>,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
    ) -> Result<JwtToken, AuthenticationError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name.clone())
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let params = GrantTypeParams {
            realm_id: realm.id,
            realm_name: realm.name,
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            code: code.clone(),
            username: username.clone(),
            password: password.clone(),
            refresh_token: token.clone(),
            redirect_uri: None,
        };

        match grant_type {
            GrantType::Code => self.using_code(client_id, code.unwrap()).await,
            GrantType::Password => {
                let username = username.ok_or(AuthenticationError::Invalid)?;
                let password = password.ok_or(AuthenticationError::Invalid)?;
                self.using_password(realm.id, client_id, username, password)
                    .await
            }
            GrantType::Credentials => self.client_credentials_strategy.execute(params).await,
            GrantType::RefreshToken => self.refresh_token_strategy.execute(params).await,
        }
    }

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
                .map_err(|_| AuthenticationError::NotFound)?;

            Ok(generate_random_string())
        } else {
            Err(AuthenticationError::InvalidPassword)
        }
    }
}
