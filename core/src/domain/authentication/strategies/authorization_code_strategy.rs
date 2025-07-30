use chrono::{TimeZone, Utc};
use tracing::error;

use crate::domain::{
    authentication::{
        entities::{AuthenticationError, JwtToken},
        ports::{AuthSessionRepository, AuthSessionService, GrantTypeStrategy},
        services::auth_session_service::AuthSessionServiceImpl,
        value_objects::GrantTypeParams,
    },
    client::{ports::ClientRepository, services::client_service::ClientServiceImpl},
    credential::{ports::CredentialRepository, services::CredentialServiceImpl},
    crypto::ports::HasherRepository,
    jwt::{
        entities::{ClaimsTyp, JwtClaim},
        ports::{JwtService, KeyStoreRepository, RefreshTokenRepository},
        services::JwtServiceImpl,
    },
    realm::ports::RealmRepository,
    user::{
        ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository, UserService},
        services::user_service::UserServiceImpl,
    },
};

#[derive(Clone)]
pub struct AuthorizationCodeStrategy<RR, K, R, C, U, UR, URA, CR, H, A>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    A: AuthSessionRepository,
{
    pub jwt_service: JwtServiceImpl<RR, K, R>,
    pub client_service: ClientServiceImpl<C, U, R>,
    pub user_service: UserServiceImpl<U, R, UR, URA>,
    pub credential_service: CredentialServiceImpl<CR, H>,
    pub auth_session_service: AuthSessionServiceImpl<A>,
}

impl<RR, K, R, C, U, UR, URA, CR, H, A> AuthorizationCodeStrategy<RR, K, R, C, U, UR, URA, CR, H, A>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    A: AuthSessionRepository,
{
    pub fn new(
        jwt_service: JwtServiceImpl<RR, K, R>,
        client_service: ClientServiceImpl<C, U, R>,
        user_service: UserServiceImpl<U, R, UR, URA>,
        credential_service: CredentialServiceImpl<CR, H>,
        auth_session_service: AuthSessionServiceImpl<A>,
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

impl<RR, K, R, C, U, UR, URA, CR, H, A> GrantTypeStrategy
    for AuthorizationCodeStrategy<RR, K, R, C, U, UR, URA, CR, H, A>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    A: AuthSessionRepository,
{
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

        let user = self.user_service.get_by_id(user_id).await.map_err(|e| {
            error!("Error retrieving user by ID: {:?}", e);
            AuthenticationError::Invalid
        })?;

        let iss = format!("{}/realms/{}", params.base_url, params.realm_name);
        let realm_audit = format!("{}-realm", params.realm_name);

        let claims = JwtClaim::new(
            user.id,
            user.username,
            iss,
            vec![realm_audit, "account".to_string()],
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
