use chrono::{TimeZone, Utc};
use tracing::info;

use crate::domain::{
    authentication::{
        entities::{AuthenticationError, JwtToken},
        ports::GrantTypeStrategy,
        value_objects::GrantTypeParams,
    },
    client::{ports::ClientRepository, services::client_service::ClientServiceImpl},
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
pub struct RefreshTokenStrategy<RR, K, R, C, U, UR, URA>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    C: ClientRepository,
{
    pub jwt_service: JwtServiceImpl<RR, K, R>,
    pub client_service: ClientServiceImpl<C, U, R>,
    pub user_service: UserServiceImpl<U, R, UR, URA>,
}

impl<RR, K, R, C, U, UR, URA> RefreshTokenStrategy<RR, K, R, C, U, UR, URA>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    C: ClientRepository,
{
    pub fn new(
        jwt_service: JwtServiceImpl<RR, K, R>,
        client_service: ClientServiceImpl<C, U, R>,
        user_service: UserServiceImpl<U, R, UR, URA>,
    ) -> Self {
        Self {
            jwt_service,
            client_service,
            user_service,
        }
    }
}

impl<RR, K, R, C, U, UR, URA> GrantTypeStrategy for RefreshTokenStrategy<RR, K, R, C, U, UR, URA>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    C: ClientRepository,
{
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

        let iss = format!("{}/realms/{}", params.base_url, params.realm_name);
        let realm_audit = format!("{}-realm", params.realm_name);

        let new_claims = JwtClaim::new(
            user.id,
            user.username,
            iss,
            vec![realm_audit, "account".to_string()],
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
