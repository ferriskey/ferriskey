use std::sync::Arc;

use chrono::{TimeZone, Utc};

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, jwt_token::JwtToken},
        ports::grant_type_strategy::{GrantTypeParams, GrantTypeStrategy},
    },
    credential::{
        ports::credential_service::CredentialService,
        services::credential_service::DefaultCredentialService,
    },
    jwt::{
        entities::jwt_claim::{ClaimsTyp, JwtClaim},
        ports::{jwt_repository::RefreshTokenRepository, jwt_service::JwtService},
        services::jwt_service::DefaultJwtService,
    },
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
};
use crate::domain::client::ports::client_service::ClientService;
use crate::domain::client::services::client_service::DefaultClientService;

#[derive(Clone)]
pub struct PasswordStrategy {
    pub jwt_service: Arc<DefaultJwtService>,
    pub user_service: Arc<DefaultUserService>,
    pub credential_service: Arc<DefaultCredentialService>,
    pub client_service: Arc<DefaultClientService>,
}

impl PasswordStrategy {
    pub fn new(
        jwt_service: Arc<DefaultJwtService>,
        user_service: Arc<DefaultUserService>,
        credential_service: Arc<DefaultCredentialService>,
        client_service: Arc<DefaultClientService>,
    ) -> Self {
        Self {
            jwt_service,
            user_service,
            credential_service,
            client_service
        }
    }
}

impl GrantTypeStrategy for PasswordStrategy {
    async fn execute(&self, params: GrantTypeParams) -> Result<JwtToken, AuthenticationError> {
        let username = params.username.ok_or(AuthenticationError::Invalid)?;
        let password = params.password.ok_or(AuthenticationError::Invalid)?;

        let client = self.client_service
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| AuthenticationError::Invalid)?;

        if client.secret != params.client_secret {
            return Err(AuthenticationError::InvalidClientSecret);
        }


        let user = self
            .user_service
            .get_by_username(username, params.realm_id)
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
            ClaimsTyp::Bearer,
            params.client_id,
            Some(user.email.clone()),
        );

        let jwt = self
            .jwt_service
            .generate_token(claims.clone(), params.realm_id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let refresh_claims = JwtClaim::new_refresh_token(
            claims.sub.clone(),
            claims.iss.clone(),
            claims.aud.clone(),
            claims.azp.clone(),
        );

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
}
