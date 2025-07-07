use chrono::{TimeZone, Utc};
use tracing::info;

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
    pub jwt_service: DefaultJwtService,
    pub client_service: DefaultClientService,
    pub user_service: DefaultUserService,
}

impl RefreshTokenStrategy {
    pub fn new(
        jwt_service: DefaultJwtService,
        client_service: DefaultClientService,
        user_service: DefaultUserService,
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
            .verify_refresh_token(refresh_token, params.realm_id)
            .await
            .map_err(|_| AuthenticationError::InvalidRefreshToken)?;

        if claims.typ != ClaimsTyp::Refresh {
            return Err(AuthenticationError::InvalidRefreshToken);
        }

        if claims.azp != params.client_id {
            info!("Invalid client_id: {:?}", claims.azp);
            return Err(AuthenticationError::InvalidRefreshToken);
        }

        let user = self
            .user_service
            .get_by_id(claims.sub)
            .await
            .map_err(|_| AuthenticationError::InvalidRefreshToken)?;

        let new_claims = JwtClaim::new(
            user.id,
            user.username,
            "http://localhost:3333/realms/master".to_string(),
            vec!["master-realm".to_string(), "account".to_string()],
            ClaimsTyp::Bearer,
            params.client_id,
            Some(user.email.clone()),
        );

        let access_token = self
            .jwt_service
            .generate_token(new_claims.clone(), params.realm_id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let refresh_claims = JwtClaim::new_refresh_token(
            new_claims.sub,
            new_claims.iss,
            new_claims.aud,
            new_claims.azp,
        );

        let refresh_token = self
            .jwt_service
            .generate_token(refresh_claims.clone(), params.realm_id)
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
                refresh_claims.jti,
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
