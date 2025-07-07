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
        entities::jwt_claim::{ClaimsTyp, JwtClaim},
        ports::{jwt_repository::RefreshTokenRepository, jwt_service::JwtService},
        services::jwt_service::DefaultJwtService,
    },
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
};
use chrono::{TimeZone, Utc};
use tracing::error;

#[derive(Clone)]
pub struct AuthorizationCodeStrategy {
    pub jwt_service: DefaultJwtService,
    pub client_service: DefaultClientService,
    pub user_service: DefaultUserService,
    pub credential_service: DefaultCredentialService,
    pub auth_session_service: DefaultAuthSessionService,
}

impl AuthorizationCodeStrategy {
    pub fn new(
        jwt_service: DefaultJwtService,
        client_service: DefaultClientService,
        user_service: DefaultUserService,
        credential_service: DefaultCredentialService,
        auth_session_service: DefaultAuthSessionService,
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
        let code = params.code.ok_or(AuthenticationError::Invalid)?;

        let auth_session = self
            .auth_session_service
            .get_by_code(code)
            .await
            .map_err(|e| {
                error!("Failed to retrieve authorization code session: {:?}", e);
                AuthenticationError::Invalid
            })?;

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
            ClaimsTyp::Bearer,
            params.client_id,
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
}
