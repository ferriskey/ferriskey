use chrono::{TimeZone, Utc};

use crate::domain::{
    authentication::{
        entities::{AuthenticationError, JwtToken},
        ports::GrantTypeStrategy,
        value_objects::GrantTypeParams,
    },
    client::{
        ports::{ClientRepository, ClientService},
        services::client_service::ClientServiceImpl,
    },
    credential::{
        ports::{CredentialRepository, CredentialService},
        services::CredentialServiceImpl,
    },
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
pub struct PasswordStrategy<RR, K, R, U, UR, URA, C, H, CR>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    C: ClientRepository,
    H: HasherRepository,
    CR: CredentialRepository,
{
    pub jwt_service: JwtServiceImpl<RR, K, R>,
    pub user_service: UserServiceImpl<U, R, UR, URA>,
    pub credential_service: CredentialServiceImpl<CR, H>,
    pub client_service: ClientServiceImpl<C, U, R>,
}

impl<RR, K, R, U, UR, URA, C, H, CR> PasswordStrategy<RR, K, R, U, UR, URA, C, H, CR>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    C: ClientRepository,
    H: HasherRepository,
    CR: CredentialRepository,
{
    pub fn new(
        jwt_service: JwtServiceImpl<RR, K, R>,
        user_service: UserServiceImpl<U, R, UR, URA>,
        credential_service: CredentialServiceImpl<CR, H>,
        client_service: ClientServiceImpl<C, U, R>,
    ) -> Self {
        Self {
            jwt_service,
            user_service,
            credential_service,
            client_service,
        }
    }
}

impl<RR, K, R, U, UR, URA, C, H, CR> GrantTypeStrategy
    for PasswordStrategy<RR, K, R, U, UR, URA, C, H, CR>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    C: ClientRepository,
    H: HasherRepository,
    CR: CredentialRepository,
{
    async fn execute(&self, params: GrantTypeParams) -> Result<JwtToken, AuthenticationError> {
        let username = params.username.ok_or(AuthenticationError::Invalid)?;
        let password = params.password.ok_or(AuthenticationError::Invalid)?;

        let client = self
            .client_service
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| AuthenticationError::Invalid)?;

        if !client.direct_access_grants_enabled {
            if params.client_secret.is_none() {
                return Err(AuthenticationError::InvalidRequest);
            }

            if client.secret != params.client_secret {
                return Err(AuthenticationError::InvalidClientSecret);
            }
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
