use std::sync::Arc;

use chrono::{TimeZone, Utc};

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, jwt_token::JwtToken},
        ports::grant_type_strategy::{GrantTypeParams, GrantTypeStrategy},
    },
    client::services::client_service::DefaultClientService,
    jwt::{
        entities::jwt_claim::{ClaimsTyp, JwtClaim},
        ports::{jwt_repository::RefreshTokenRepository, jwt_service::JwtService},
        services::jwt_service::DefaultJwtService,
    },
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
};

#[derive(Clone)]
pub struct RefreshTokenStrategy {
    pub jwt_service: Arc<DefaultJwtService>,
    pub client_service: Arc<DefaultClientService>,
    pub user_service: Arc<DefaultUserService>,
}

impl RefreshTokenStrategy {
    pub fn new(
        jwt_service: Arc<DefaultJwtService>,
        client_service: Arc<DefaultClientService>,
        user_service: Arc<DefaultUserService>,
    ) -> Self {
        Self {
            jwt_service,
            client_service,
            user_service,
        }
    }
}

impl GrantTypeStrategy for RefreshTokenStrategy {
    async fn execute(&self, params: GrantTypeParams) -> Result<JwtToken, AuthenticationError> {
        let refresh_token = params.refresh_token.ok_or(AuthenticationError::Invalid)?;

        let claims = self
            .jwt_service
            .verify_token(refresh_token)
            .await
            .map_err(|_| AuthenticationError::InvalidRefreshToken)?;

        if claims.typ != ClaimsTyp::Refresh {
            return Err(AuthenticationError::InvalidRefreshToken);
        }

        let user = self
            .user_service
            .get_by_id(claims.sub)
            .await
            .map_err(|_| AuthenticationError::InvalidRefreshToken)?;

        let claims = JwtClaim::new(
            user.id,
            user.username,
            "http://localhost:3333/realms/master".to_string(),
            vec!["master-realm".to_string(), "account".to_string()],
            ClaimsTyp::Bearer,
            params.client_id,
        );

        let access_token = self
            .jwt_service
            .generate_token(claims.clone())
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user.id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        self.jwt_service
            .refresh_token_repository
            .delete(claims.jti)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        self.jwt_service
            .refresh_token_repository
            .create(
                claims.jti,
                user.id,
                Some(Utc.timestamp_opt(refresh_token.expires_at, 0).unwrap()),
            )
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        Ok(JwtToken::new(
            access_token.token,
            "Bearer".to_string(),
            refresh_token.token,
            3600,
            "id_token".to_string(),
        ))
    }
}
