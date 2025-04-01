use std::sync::Arc;

use crate::domain::realm::ports::RealmService;

use super::entities::error::AuthenticationError;
use super::entities::model::{GrantType, JwtToken};
use super::ports::{AuthenticationRepository, AuthenticationService};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticationServiceImpl<A: Clone + Send + Sync + 'static, R>
where
    A: AuthenticationRepository,
    R: RealmService,
{
    pub authentication_repository: A,
    pub realm_service: Arc<R>,
}

impl<A, R> AuthenticationServiceImpl<A, R>
where
    A: AuthenticationRepository,
    R: RealmService,
{
    pub fn new(authentication_repository: A, realm_service: Arc<R>) -> Self {
        Self {
            authentication_repository,
            realm_service,
        }
    }
}

impl<A, R> AuthenticationService for AuthenticationServiceImpl<A, R>
where
    A: AuthenticationRepository,
    R: RealmService,
{
    async fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> Result<JwtToken, AuthenticationError> {
        self.authentication_repository
            .using_code(client_id, code)
            .await
    }

    async fn using_password(
        &self,
        user_id: Uuid,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError> {
        self.authentication_repository
            .using_password(user_id, username, password)
            .await
    }

    async fn using_credentials(
        &self,
        realm_id: Uuid,
        client_id: String,
        client_secret: String,
    ) -> Result<JwtToken, AuthenticationError> {
        let client = self
            .authentication_repository
            .using_credentials(realm_id, client_id, client_secret)
            .await;

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
                self.using_password(realm.id, username.unwrap(), password.unwrap())
                    .await
            }
            GrantType::Credentials => {
                self.using_credentials(realm.id, client_id, client_secret.unwrap())
                    .await
            }
        }
    }
}
