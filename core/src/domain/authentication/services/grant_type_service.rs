use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{AuthenticationError, GrantType, JwtToken},
        ports::{AuthSessionRepository, GrantTypeService, GrantTypeStrategy},
        value_objects::GrantTypeParams,
    },
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::{
        entities::ClaimsTyp,
        ports::{KeyStoreRepository, RefreshTokenRepository},
    },
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    trident::ports::RecoveryCodeRepository,
    user::ports::UserRepository,
    user::ports::{UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::{WebhookNotifierRepository, WebhookRepository},
};

pub struct GenerateTokenInput {
    pub base_url: String,
    pub realm_name: String,
    pub user_id: Uuid,
    pub username: String,
    pub client_id: String,
    pub email: String,
    pub realm_id: Uuid,
}

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC> GrantTypeService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    WN: WebhookNotifierRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
{
    async fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> Result<JwtToken, AuthenticationError> {
        match grant_type {
            GrantType::Code => self
                .authorization_code(params)
                .await
                .map_err(|_| AuthenticationError::InternalServerError),
            GrantType::Password => self
                .password(params)
                .await
                .map_err(|_| AuthenticationError::InternalServerError),
            GrantType::Credentials => self
                .client_credential(params)
                .await
                .map_err(|_| AuthenticationError::InternalServerError),
            GrantType::RefreshToken => self
                .refresh_token(params)
                .await
                .map_err(|_| AuthenticationError::InternalServerError),
        }
    }
}

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC> GrantTypeStrategy
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    WN: WebhookNotifierRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
{
    async fn authorization_code(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let code = params.code.ok_or(CoreError::InternalServerError)?;

        let auth_session = self
            .auth_session_repository
            .get_by_code(code)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::NotFound)?;

        let user_id = auth_session.user_id.ok_or(CoreError::NotFound)?;

        let user = self
            .user_repository
            .get_by_id(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let (jwt, refresh_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
            })
            .await?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            3600,
            "id_token".to_string(),
        ))
    }

    async fn client_credential(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if client.secret != params.client_secret {
            return Err(CoreError::InvalidClientSecret);
        }

        let user = self
            .user_repository
            .get_by_client_id(client.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let (jwt, refresh_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
            })
            .await?;
        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            3600,
            "id_token".to_string(),
        ))
    }

    async fn password(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let username = params.username.ok_or(CoreError::InternalServerError)?;
        let password = params.password.ok_or(CoreError::InternalServerError)?;

        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if !client.direct_access_grants_enabled {
            if params.client_secret.is_none() {
                return Err(CoreError::InternalServerError);
            }

            if client.secret != params.client_secret {
                return Err(CoreError::InvalidClientSecret);
            }
        }

        let user = self
            .user_repository
            .get_by_username(username, params.realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let credential = self.verify_password(user.id, password).await;

        let is_valid = match credential {
            Ok(is_valid) => is_valid,
            Err(_) => return Err(CoreError::Invalid),
        };

        if !is_valid {
            return Err(CoreError::Invalid);
        }

        let (jwt, refresh_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
            })
            .await?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            3600,
            "id_token".to_string(),
        ))
    }

    async fn refresh_token(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let refresh_token = params.refresh_token.ok_or(CoreError::InvalidRefreshToken)?;

        let claims = self
            .verify_refresh_token(refresh_token, params.realm_id)
            .await?;

        if claims.typ != ClaimsTyp::Refresh {
            return Err(CoreError::InvalidToken);
        }

        if claims.azp != params.client_id {
            tracing::warn!("invalid client id: {:?}", claims.azp);
            return Err(CoreError::InvalidToken);
        }

        let user = self
            .user_repository
            .get_by_id(claims.sub)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let (jwt, refresh_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
            })
            .await?;

        self.refresh_token_repository
            .delete(claims.jti)
            .await
            .map_err(|_| CoreError::InternalServerError)?;
        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            3600,
            "id_token".to_string(),
        ))
    }
}
