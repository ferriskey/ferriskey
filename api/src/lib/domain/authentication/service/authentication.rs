use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, grant_type::GrantType, jwt_token::JwtToken},
        ports::{auth_session::AuthSessionService, authentication::AuthenticationService},
    },
    client::ports::client_service::ClientService,
    credential::ports::credential_service::CredentialService,
    jwt::{entities::jwt_claim::JwtClaim, ports::jwt_service::JwtService},
    realm::ports::realm_service::RealmService,
    user::ports::user_service::UserService,
    utils::generate_random_string,
};

#[derive(Clone)]
pub struct AuthenticationServiceImpl<R, C, CR, U>
where
    R: RealmService,
    C: ClientService,
    CR: CredentialService,
    U: UserService,
{
    pub realm_service: Arc<R>,
    pub client_service: Arc<C>,
    pub credential_service: Arc<CR>,
    pub user_service: Arc<U>,
    pub jwt_service: Arc<dyn JwtService>,
    pub auth_session_service: Arc<dyn AuthSessionService>,
}

impl<R, C, CR, U> AuthenticationServiceImpl<R, C, CR, U>
where
    R: RealmService,
    C: ClientService,
    CR: CredentialService,
    U: UserService,
{
    pub fn new(
        realm_service: Arc<R>,
        client_service: Arc<C>,
        credential_service: Arc<CR>,
        user_service: Arc<U>,
        jwt_service: Arc<dyn JwtService>,
        auth_session_service: Arc<dyn AuthSessionService>,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            credential_service,
            user_service,
            jwt_service,
            auth_session_service,
        }
    }
}

impl<R, C, CR, U> AuthenticationService for AuthenticationServiceImpl<R, C, CR, U>
where
    R: RealmService,
    C: ClientService,
    CR: CredentialService,
    U: UserService,
{
    async fn using_code(
        &self,
        _client_id: String,
        _code: String,
    ) -> Result<JwtToken, AuthenticationError> {
        todo!("using_code")
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
            user.id.to_string(),
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

    async fn using_credentials(
        &self,
        realm_id: Uuid,
        client_id: String,
        _client_secret: String,
    ) -> Result<JwtToken, AuthenticationError> {
        let client = self
            .client_service
            .get_by_client_id(client_id, realm_id)
            .await
            .map_err(|_| AuthenticationError::Invalid);

        match client {
            Ok(client) => {
                info!("success to login with client: {:?}", client.name);

                Ok(JwtToken::new(
                    client.secret.unwrap(),
                    "Bearer".to_string(),
                    "8xLOxBtZp8".to_string(),
                    3600,
                    "id_token".to_string(),
                ))
            }
            Err(error) => Err(error),
        }
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
    ) -> Result<JwtToken, AuthenticationError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        match grant_type {
            GrantType::Code => self.using_code(client_id, code.unwrap()).await,
            GrantType::Password => {
                let username = username.ok_or(AuthenticationError::Invalid)?;
                let password = password.ok_or(AuthenticationError::Invalid)?;
                self.using_password(realm.id, client_id, username, password)
                    .await
            }
            GrantType::Credentials => {
                self.using_credentials(realm.id, client_id, client_secret.unwrap())
                    .await
            }
        }
    }

    async fn using_session_code(
        &self,
        realm_name: String,
        client_id: Uuid,
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
            .get_by_client_id(client_id.to_string(), realm.id)
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
