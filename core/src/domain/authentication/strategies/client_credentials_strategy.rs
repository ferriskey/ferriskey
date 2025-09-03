use chrono::{TimeZone, Utc};

use crate::domain::{
    authentication::{
        entities::{AuthenticationError, JwtToken},
        ports::GrantTypeStrategy,
        value_objects::GrantTypeParams,
    },
    client::{
        ports::{ClientRepository, OldClientService},
        services::client_service::ClientServiceImpl,
    },
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
pub struct ClientCredentialsStrategy<C, U, R, UR, URA, RR, K>
where
    C: ClientRepository,
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
{
    pub client_service: ClientServiceImpl<C, U, R>,
    pub user_service: UserServiceImpl<U, R, UR, URA>,
    pub jwt_service: JwtServiceImpl<RR, K, R>,
}

impl<C, U, R, UR, URA, RR, K> ClientCredentialsStrategy<C, U, R, UR, URA, RR, K>
where
    C: ClientRepository,
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
{
    pub fn new(
        client_service: ClientServiceImpl<C, U, R>,
        user_service: UserServiceImpl<U, R, UR, URA>,
        jwt_service: JwtServiceImpl<RR, K, R>,
    ) -> Self {
        Self {
            client_service,
            user_service,
            jwt_service,
        }
    }
}

impl<C, U, R, UR, URA, RR, K> GrantTypeStrategy
    for ClientCredentialsStrategy<C, U, R, UR, URA, RR, K>
where
    C: ClientRepository,
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
{
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

                let iss = format!("{}/realms/{}", params.base_url, params.realm_name);
                let realm_audit = format!("{}-realm", params.realm_name);

                let claims = JwtClaim::new(
                    user.id,
                    user.username,
                    iss,
                    vec![realm_audit, "account".to_string()],
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
