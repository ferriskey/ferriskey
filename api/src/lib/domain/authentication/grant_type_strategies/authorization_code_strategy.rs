use std::sync::Arc;

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, jwt_token::JwtToken},
        ports::{
            auth_session::AuthSessionService,
            grant_type_strategy::{GrantTypeParams, GrantTypeStrategy},
        },
        service::auth_session::DefaultAuthSessionService,
    },
    client::services::client_service::DefaultClientService,
    credential::services::credential_service::DefaultCredentialService,
    jwt::{
        entities::jwt_claim::JwtClaim, ports::jwt_service::JwtService,
        services::jwt_service::DefaultJwtService,
    },
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
};

#[derive(Clone)]
pub struct AuthorizationCodeStrategy {
    pub jwt_service: Arc<DefaultJwtService>,
    pub client_service: Arc<DefaultClientService>,
    pub user_service: Arc<DefaultUserService>,
    pub credential_service: Arc<DefaultCredentialService>,
    pub auth_session_service: Arc<DefaultAuthSessionService>,
}

impl AuthorizationCodeStrategy {
    pub fn new(
        jwt_service: Arc<DefaultJwtService>,
        client_service: Arc<DefaultClientService>,
        user_service: Arc<DefaultUserService>,
        credential_service: Arc<DefaultCredentialService>,
        auth_session_service: Arc<DefaultAuthSessionService>,
    ) -> Self {
        Self {
            jwt_service,
            client_service,
            user_service,
            credential_service,
            auth_session_service,
        }
    }
}

impl GrantTypeStrategy for AuthorizationCodeStrategy {
    async fn execute(&self, params: GrantTypeParams) -> Result<JwtToken, AuthenticationError> {
        let auth_session = self
            .auth_session_service
            .get_by_code(params.code.unwrap())
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
            params.client_id,
        );

        let jwt = self
            .jwt_service
            .generate_token(claims)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user.id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            3600,
            "id_token".to_string(),
        ))
    }
}
