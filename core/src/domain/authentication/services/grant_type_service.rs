use chrono::{TimeZone, Utc};
use jsonwebtoken::{Algorithm, Header, Validation};
use uuid::Uuid;

use crate::{
    domain::{
        authentication::{
            entities::{AuthenticationError, GrantType, JwtToken},
            ports::{AuthSessionRepository, GrantTypeService, GrantTypeStrategy},
            value_objects::GrantTypeParams,
        },
        client::ports::ClientRepository,
        common::entities::app_errors::CoreError,
        credential::ports::CredentialRepository,
        crypto::ports::HasherRepository,
        jwt::{
            entities::{ClaimsTyp, Jwt, JwtClaim},
            ports::{KeyStoreRepository, RefreshTokenRepository},
        },
        user::ports::UserRepository,
    },
    infrastructure::{
        auth_session::AuthSessionRepoAny, client::repositories::ClientRepoAny,
        credential::CredentialRepoAny, hasher::HasherRepoAny, jwt::KeyStoreRepoAny,
        refresh_token::RefreshTokenRepoAny, user::UserRepoAny,
    },
};

#[derive(Clone)]
pub struct GrantTypeStrategies {
    credential_repository: CredentialRepoAny,
    hasher_repository: HasherRepoAny,
    auth_session_repository: AuthSessionRepoAny,
    user_repository: UserRepoAny,
    keystore_repository: KeyStoreRepoAny,
    refresh_token_repository: RefreshTokenRepoAny,
    client_repository: ClientRepoAny,
}

struct GenerateTokenInput {
    base_url: String,
    realm_name: String,
    user_id: Uuid,
    username: String,
    client_id: String,
    email: String,
    realm_id: Uuid,
}

impl GrantTypeStrategies {
    pub fn new(
        credential_repository: CredentialRepoAny,
        hasher_repository: HasherRepoAny,
        auth_session_repository: AuthSessionRepoAny,
        user_repository: UserRepoAny,
        keystore_repository: KeyStoreRepoAny,
        refresh_token_repository: RefreshTokenRepoAny,
        client_repository: ClientRepoAny,
    ) -> Self {
        Self {
            credential_repository,
            hasher_repository,
            auth_session_repository,
            user_repository,
            keystore_repository,
            refresh_token_repository,
            client_repository,
        }
    }

    async fn verify_password(&self, user_id: Uuid, password: String) -> Result<bool, CoreError> {
        let credential = self
            .credential_repository
            .get_password_credential(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let salt = credential.salt.ok_or(CoreError::InternalServerError)?;

        let is_valid = self
            .hasher_repository
            .verify_password(
                &password,
                &credential.secret_data,
                &credential.credential_data,
                &salt,
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(is_valid)
    }

    async fn generate_token(&self, claims: JwtClaim, realm_id: Uuid) -> Result<Jwt, CoreError> {
        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let header = Header::new(jsonwebtoken::Algorithm::RS256);
        let token =
            jsonwebtoken::encode(&header, &claims, &jwt_key_pair.encoding_key).map_err(|e| {
                tracing::error!("JWT generation error: {}", e);

                CoreError::TokenGenerationError(e.to_string())
            })?;

        let exp = claims.exp.unwrap_or(0);

        Ok(Jwt {
            token,
            expires_at: exp,
        })
    }

    async fn create_jwt(&self, input: GenerateTokenInput) -> Result<(Jwt, Jwt), CoreError> {
        let iss = format!("{}/realms/{}", input.base_url, input.realm_name);
        let realm_audit = format!("{}-realm", input.realm_name);

        let claims = JwtClaim::new(
            input.user_id,
            input.username,
            iss,
            vec![realm_audit, "account".to_string()],
            ClaimsTyp::Bearer,
            input.client_id,
            Some(input.email),
        );

        let jwt = self.generate_token(claims.clone(), input.realm_id).await?;

        let refresh_claims =
            JwtClaim::new_refresh_token(claims.sub, claims.iss, claims.aud, claims.azp);

        let refresh_token = self
            .generate_token(refresh_claims.clone(), input.realm_id)
            .await?;

        self.refresh_token_repository
            .create(
                refresh_claims.jti,
                input.user_id,
                Some(Utc.timestamp_opt(refresh_token.expires_at, 0).unwrap()),
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok((jwt, refresh_token))
    }

    pub async fn verify_token(&self, token: String, realm_id: Uuid) -> Result<JwtClaim, CoreError> {
        let mut validation = Validation::new(Algorithm::RS256);

        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        validation.validate_aud = false;
        let token_data =
            jsonwebtoken::decode::<JwtClaim>(&token, &jwt_key_pair.decoding_key, &validation)
                .map_err(|e| CoreError::TokenValidationError(e.to_string()))?;

        let current_time = chrono::Utc::now().timestamp();

        if let Some(exp) = token_data.claims.exp
            && exp < current_time
        {
            return Err(CoreError::ExpiredToken);
        }

        Ok(token_data.claims)
    }

    pub async fn verify_refresh_token(
        &self,
        token: String,
        realm_id: Uuid,
    ) -> Result<JwtClaim, CoreError> {
        let claims = self.verify_token(token, realm_id).await?;

        let refresh_token = self
            .refresh_token_repository
            .get_by_jti(claims.jti)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if refresh_token.revoked {
            return Err(CoreError::ExpiredToken);
        }

        if let Some(expires_at) = refresh_token.expires_at
            && expires_at < chrono::Utc::now()
        {
            return Err(CoreError::ExpiredToken);
        }

        Ok(claims)
    }
}

impl GrantTypeService for GrantTypeStrategies {
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

impl GrantTypeStrategy for GrantTypeStrategies {
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

// {
//     async fn authenticate_with_grant_type(
//         &self,
//         grant_type: GrantType,
//         params: GrantTypeParams,
//     ) -> Result<JwtToken, AuthenticationError> {
//         match grant_type {
//             GrantType::Code => self.authorization_code_strategy.execute(params).await,
//             GrantType::Password => self.password_strategy.execute(params).await,
//             GrantType::Credentials => self.client_credentials_strategy.execute(params).await,
//             GrantType::RefreshToken => self.refresh_token_strategy.execute(params).await,
//         }
//     }
// }
