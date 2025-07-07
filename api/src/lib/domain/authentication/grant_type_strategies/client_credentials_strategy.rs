use chrono::{TimeZone, Utc};

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, jwt_token::JwtToken},
        ports::grant_type_strategy::{GrantTypeParams, GrantTypeStrategy},
    },
    client::{
        ports::client_service::ClientService, services::client_service::DefaultClientService,
    },
    jwt::{
        entities::jwt_claim::{ClaimsTyp, JwtClaim},
        ports::{jwt_repository::RefreshTokenRepository, jwt_service::JwtService},
        services::jwt_service::DefaultJwtService,
    },
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
};

#[derive(Clone)]
pub struct ClientCredentialsStrategy {
    pub client_service: DefaultClientService,
    pub user_service: DefaultUserService,
    pub jwt_service: DefaultJwtService,
}

impl ClientCredentialsStrategy {
    pub fn new(
        client_service: DefaultClientService,
        user_service: DefaultUserService,
        jwt_service: DefaultJwtService,
    ) -> Self {
        Self {
            client_service,
            user_service,
            jwt_service,
        }
    }
}

impl GrantTypeStrategy for ClientCredentialsStrategy {
    async fn execute(&self, params: GrantTypeParams) -> Result<JwtToken, AuthenticationError> {
        let client = self
            .client_service
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| AuthenticationError::Invalid);

        match client {
            Ok(client) => {
                if client.secret != params.client_secret {
                    return Err(AuthenticationError::InvalidClientSecret);
                }

                let user = self
                    .user_service
                    .get_by_client_id(client.id)
                    .await
                    .map_err(|_| AuthenticationError::ServiceAccountNotFound)?;

                let claims = JwtClaim::new(
                    user.id,
                    user.username,
                    "http://localhost:3333/realms/master".to_string(),
                    vec!["master-realm".to_string(), "account".to_string()],
                    ClaimsTyp::Bearer,
                    params.client_id.clone(),
                    Some(user.email.clone()),
                );

                let jwt = self
                    .jwt_service
                    .generate_token(claims.clone(), params.realm_id)
                    .await
                    .map_err(|_| AuthenticationError::InternalServerError)?;

                let refresh_claims =
                    JwtClaim::new_refresh_token(claims.sub, claims.iss, claims.aud, claims.azp);

                let refresh_token = self
                    .jwt_service
                    .generate_token(refresh_claims.clone(), params.realm_id)
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
                    jwt.token,
                    "Bearer".to_string(),
                    refresh_token.token,
                    3600,
                    "id_token".to_string(),
                ))
            }
            Err(error) => Err(error),
        }
    }
}
