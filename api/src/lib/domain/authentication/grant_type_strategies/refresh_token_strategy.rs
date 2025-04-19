use std::sync::Arc;

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, jwt_token::JwtToken},
        ports::grant_type_strategy::{GrantTypeParams, GrantTypeStrategy},
    },
    client::{
        ports::client_service::ClientService, services::client_service::DefaultClientService,
    },
    jwt::{ports::jwt_service::JwtService, services::jwt_service::DefaultJwtService},
};

#[derive(Clone)]
pub struct RefreshTokenStrategy {
    pub jwt_service: Arc<DefaultJwtService>,
    pub client_service: Arc<DefaultClientService>,
}

impl RefreshTokenStrategy {
    pub fn new(
        jwt_service: Arc<DefaultJwtService>,
        client_service: Arc<DefaultClientService>,
    ) -> Self {
        Self {
            jwt_service,
            client_service,
        }
    }
}

impl GrantTypeStrategy for RefreshTokenStrategy {
    async fn execute(&self, params: GrantTypeParams) -> Result<JwtToken, AuthenticationError> {
        let _ = self
            .client_service
            .get_by_client_id(params.client_id, params.realm_id)
            .await
            .map_err(|_| AuthenticationError::InvalidClient)?;

        let token = params.refresh_token.ok_or(AuthenticationError::Invalid)?;
        let claims = self
            .jwt_service
            .verify_token(token)
            .await
            .map_err(|_| AuthenticationError::InvalidRefreshToken)?;

        let jwt = self
            .jwt_service
            .generate_token(claims)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            "8xLOxBtZp8".to_string(),
            3600,
            "id_token".to_string(),
        ))
    }
}
