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


#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;

    mock! {
        pub GrantTypeStrategies {}
        impl Clone for GrantTypeStrategies { fn clone(&self) -> Self; }
        impl GrantTypeService for GrantTypeStrategies {
            fn authenticate_with_grant_type(&self, grant_type: GrantType, params: GrantTypeParams)
                -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
        }
        impl GrantTypeStrategy for GrantTypeStrategies {
            fn authorization_code(&self, params: GrantTypeParams) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
            fn client_credential(&self, params: GrantTypeParams) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
            fn refresh_token(&self, params: GrantTypeParams) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
            fn password(&self, params: GrantTypeParams) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
        }
    }
}


#[cfg(test)]
mod tests_grant_type {
    use crate::domain::credential::entities::Credential;
use crate::domain::credential::entities::CredentialConfig;
    use crate::domain::crypto::ports::HasherRepository;
    use crate::domain::credential::entities::CredentialData;
    use crate::infrastructure::auth_session::AuthSessionRepoAny;
    use crate::domain::authentication::entities::AuthSession;
    use crate::domain::authentication::ports::GrantTypeStrategy;
    use std::future::ready;

    use super::GrantTypeStrategies;
    use crate::domain::authentication::value_objects::GrantTypeParams;
    use crate::domain::client::entities::{Client, ClientConfig};
    use crate::domain::common::entities::app_errors::CoreError;
    use crate::domain::jwt::entities::{ClaimsTyp, JwtClaim, JwtKeyPair, RefreshToken};
    use crate::domain::jwt::ports::test::{
        get_mock_key_store_repository_with_clone_expectations,
        get_mock_refresh_token_repository_with_clone_expectations,
    };
    use crate::domain::user::entities::{User, UserConfig};
    use crate::infrastructure::client::repositories::ClientRepoAny;
    use crate::infrastructure::jwt::KeyStoreRepoAny;
    use crate::infrastructure::refresh_token::RefreshTokenRepoAny;
    use crate::infrastructure::repositories::test::build_repos_mock;
    use crate::infrastructure::repositories::RepoBundle;
    use crate::infrastructure::user::UserRepoAny;
    use uuid::Uuid;
    use crate::domain::authentication::ports::test::get_mock_auth_session_repository_with_clone_expectations;
    use crate::domain::credential::ports::test::get_mock_credential_repository_with_clone_expectations;

    fn build_strategies(repos: RepoBundle) -> GrantTypeStrategies {
        GrantTypeStrategies::new(
            repos.credential_repository,
            repos.hasher_repository,
            repos.auth_session_repository,
            repos.user_repository,
            repos.keystore_repository,
            repos.refresh_token_repository,
            repos.client_repository,
        )
    }

    #[tokio::test]
    async fn client_credentials_success_generates_tokens() {
        // Arrange
        let mut client_repo = if let ClientRepoAny::Mock(m) = build_repos_mock().client_repository {
            m
        } else {
            unreachable!()
        };
        let mut user_repo = if let UserRepoAny::Mock(m) = build_repos_mock().user_repository {
            m
        } else {
            unreachable!()
        };
        let mut keystore_repo = get_mock_key_store_repository_with_clone_expectations();
        let mut refresh_repo = get_mock_refresh_token_repository_with_clone_expectations();

        let realm_id = Uuid::new_v4();
        let client = Client::new(ClientConfig {
            realm_id,
            name: "svc".to_string(),
            client_id: "client123".to_string(),
            secret: Some("secret".to_string()),
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: true,
            client_type: "service".to_string(),
            direct_access_grants_enabled: Some(true),
        });
        let user = User::new(UserConfig {
            realm_id,
            client_id: Some(client.id),
            username: "svc-account".to_string(),
            firstname: "Svc".to_string(),
            lastname: "Account".to_string(),
            email: "svc@example.com".to_string(),
            email_verified: true,
            enabled: true,
        });

        // client and user lookups
        let client_clone = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |_cid: String, _rid: Uuid| Box::pin(ready(Ok(client_clone.clone()))));
        let user_clone = user.clone();
        user_repo
            .expect_get_by_client_id()
            .returning(move |cid: Uuid| {
                assert_eq!(cid, client.id);
                Box::pin(ready(Ok(user_clone.clone())))
            });

        // keys and refresh token persistence
        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm_id, Uuid::new_v4()).unwrap();
        let keypair_for_mock = keypair.clone();
        keystore_repo
            .expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(keypair_for_mock.clone()))));

        let user_id_for_assert = user.id;
        refresh_repo
            .expect_create()
            .returning(move |_jti: Uuid, uid: Uuid, _exp| {
                assert_eq!(uid, user_id_for_assert);
                let rt = RefreshToken::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    uid,
                    false,
                    None,
                    chrono::Utc::now(),
                );
                Box::pin(ready(Ok(rt)))
            });

        let mut repos = build_repos_mock();
        repos.client_repository = ClientRepoAny::Mock(client_repo);
        repos.user_repository = UserRepoAny::Mock(user_repo);
        repos.keystore_repository = KeyStoreRepoAny::Mock(keystore_repo);
        repos.refresh_token_repository = RefreshTokenRepoAny::Mock(refresh_repo);

        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id,
            base_url: "https://auth.local".to_string(),
            realm_name: "master".to_string(),
            client_id: client.client_id.clone(),
            client_secret: Some("secret".to_string()),
            code: None,
            username: None,
            password: None,
            refresh_token: None,
            redirect_uri: None,
        };

        // Act
        let out = strategies.client_credential(params).await;

        // Assert
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn client_credentials_invalid_secret_errors() {
        let mut client_repo = if let ClientRepoAny::Mock(m) = build_repos_mock().client_repository {
            m
        } else {
            unreachable!()
        };
        let realm_id = Uuid::new_v4();
        let client = Client::new(ClientConfig {
            realm_id,
            name: "svc".to_string(),
            client_id: "client123".to_string(),
            secret: Some("right".to_string()),
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: true,
            client_type: "service".to_string(),
            direct_access_grants_enabled: Some(true),
        });
        let client_clone = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |_cid: String, _rid: Uuid| Box::pin(ready(Ok(client_clone.clone()))));
        let mut repos = build_repos_mock();
        repos.client_repository = ClientRepoAny::Mock(client_repo);
        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id,
            base_url: "https://auth.local".to_string(),
            realm_name: "master".to_string(),
            client_id: client.client_id.clone(),
            client_secret: Some("wrong".to_string()),
            code: None,
            username: None,
            password: None,
            refresh_token: None,
            redirect_uri: None,
        };
        let err = strategies.client_credential(params).await.err().unwrap();
        assert!(matches!(err, CoreError::InvalidClientSecret));
    }

    #[tokio::test]
    async fn password_requires_secret_when_direct_grants_disabled() {
        let mut client_repo = if let ClientRepoAny::Mock(m) = build_repos_mock().client_repository {
            m
        } else {
            unreachable!()
        };
        let realm_id = Uuid::new_v4();
        let client = Client::new(ClientConfig {
            realm_id,
            name: "app".to_string(),
            client_id: "client123".to_string(),
            secret: Some("secret".to_string()),
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: false,
            client_type: "confidential".to_string(),
            direct_access_grants_enabled: Some(false),
        });
        let client_clone = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |_cid: String, _rid: Uuid| Box::pin(ready(Ok(client_clone.clone()))));

        let mut repos = build_repos_mock();
        repos.client_repository = ClientRepoAny::Mock(client_repo);
        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id,
            base_url: "https://auth.local".to_string(),
            realm_name: "master".to_string(),
            client_id: client.client_id.clone(),
            client_secret: None, // missing secret
            code: None,
            username: Some("alice".to_string()),
            password: Some("password".to_string()),
            refresh_token: None,
            redirect_uri: None,
        };
        let err = strategies.password(params).await.err().unwrap();
        assert!(matches!(err, CoreError::InternalServerError));
    }

    #[tokio::test]
    async fn verify_refresh_token_revoked_errors() {
        let realm_id = Uuid::new_v4();
        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm_id, Uuid::new_v4()).unwrap();

        let mut repos = build_repos_mock();

        // Override keystore
        let mut ks = get_mock_key_store_repository_with_clone_expectations();
        let keypair_for_mock = keypair.clone();
        ks.expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(keypair_for_mock.clone()))));
        repos.keystore_repository = KeyStoreRepoAny::Mock(ks);

        // Override refresh repo
        let mut rr = get_mock_refresh_token_repository_with_clone_expectations();
        rr.expect_get_by_jti().returning(move |_jti: Uuid| {
            let rt = RefreshToken::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                true,
                None,
                chrono::Utc::now(),
            );
            Box::pin(ready(Ok(rt)))
        });
        repos.refresh_token_repository = RefreshTokenRepoAny::Mock(rr);

        let strategies = build_strategies(repos);

        // Build a valid bearer token that is not expired
        let mut claims = JwtClaim::new(
            Uuid::new_v4(),
            "user".to_string(),
            "iss".to_string(),
            vec!["aud".to_string()],
            ClaimsTyp::Bearer,
            "azp".to_string(),
            Some("u@example.com".to_string()),
        );
        claims.exp = Some(chrono::Utc::now().timestamp() + 3600);
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &keypair.encoding_key,
        )
        .unwrap();

        let err = strategies
            .verify_refresh_token(token, realm_id)
            .await
            .err()
            .unwrap();
        assert!(matches!(err, CoreError::ExpiredToken));
    }

    #[tokio::test]
    async fn verify_token_expired_errors() {
        let realm_id = Uuid::new_v4();
        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm_id, Uuid::new_v4()).unwrap();

        let mut repos = build_repos_mock();
        // Override keystore
        if let KeyStoreRepoAny::Mock(mut ks) = repos.keystore_repository.clone() {
            let keypair_for_mock = keypair.clone();
            ks.expect_get_or_generate_key()
                .returning(move |_rid: Uuid| Box::pin(ready(Ok(keypair_for_mock.clone()))));
            repos.keystore_repository = KeyStoreRepoAny::Mock(ks);
        }

        let strategies = build_strategies(repos);

        let mut claims = JwtClaim::new(
            Uuid::new_v4(),
            "user".to_string(),
            "iss".to_string(),
            vec!["aud".to_string()],
            ClaimsTyp::Bearer,
            "azp".to_string(),
            Some("u@example.com".to_string()),
        );
        // Set expiration in the past
        claims.exp = Some(chrono::Utc::now().timestamp() - 10);
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &keypair.encoding_key,
        )
        .unwrap();

        let err = strategies.verify_token(token, realm_id).await.err().unwrap();
        assert!(matches!(err, CoreError::ExpiredToken));
    }

    #[tokio::test]
    async fn authorization_code_success_returns_tokens() {
        // Arrange
        let mut repos = build_repos_mock();
        let mut session_repo = get_mock_auth_session_repository_with_clone_expectations();
        let mut user_repo = if let UserRepoAny::Mock(m) = repos.user_repository { m } else { unreachable!() };
        let mut keystore_repo = get_mock_key_store_repository_with_clone_expectations();
        let mut refresh_repo = get_mock_refresh_token_repository_with_clone_expectations();

        let realm_id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let user = User::new(UserConfig {
            realm_id,
            client_id: None,
            username: "alice".to_string(),
            firstname: "Alice".to_string(),
            lastname: "Doe".to_string(),
            email: "alice@example.com".to_string(),
            email_verified: true,
            enabled: true,
        });
        let session = AuthSession {
            id: Uuid::new_v4(),
            realm_id,
            client_id,
            redirect_uri: "https://example.com/cb".into(),
            response_type: "code".into(),
            scope: "openid".into(),
            state: None,
            nonce: None,
            user_id: Some(user.id),
            code: Some("abc".into()),
            authenticated: true,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now(),
        };

        session_repo
            .expect_get_by_code()
            .returning(move |c: String| {
                assert_eq!(c, "abc");
                Box::pin(ready(Ok(Some(session.clone()))))
            });
        let user_for_mock = user.clone();
        let user_id_for_assert = user.id;
        user_repo
            .expect_get_by_id()
            .returning(move |uid: Uuid| {
                assert_eq!(uid, user_id_for_assert);
                Box::pin(ready(Ok(user_for_mock.clone())))
            });

        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm_id, Uuid::new_v4()).unwrap();
        let keypair_for_mock = keypair.clone();
        keystore_repo
            .expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(keypair_for_mock.clone()))));

        let user_id_for_assert = user.id;
        refresh_repo
            .expect_create()
            .returning(move |_jti: Uuid, uid: Uuid, _exp| {
                assert_eq!(uid, user_id_for_assert);
                let rt = RefreshToken::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    uid,
                    false,
                    None,
                    chrono::Utc::now(),
                );
                Box::pin(ready(Ok(rt)))
            });

        repos.auth_session_repository = AuthSessionRepoAny::Mock(session_repo);
        repos.user_repository = UserRepoAny::Mock(user_repo);
        repos.keystore_repository = KeyStoreRepoAny::Mock(keystore_repo);
        repos.refresh_token_repository = RefreshTokenRepoAny::Mock(refresh_repo);

        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id,
            base_url: "https://auth.local".to_string(),
            realm_name: "master".to_string(),
            client_id: "client123".to_string(),
            client_secret: None,
            code: Some("abc".to_string()),
            username: None,
            password: None,
            refresh_token: None,
            redirect_uri: None,
        };

        // Act
        let out = strategies.authorization_code(params).await;

        // Assert
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn authorization_code_missing_code_errors() {
        let strategies = build_strategies(build_repos_mock());
        let params = GrantTypeParams {
            realm_id: Uuid::new_v4(),
            base_url: "https://auth.local".into(),
            realm_name: "master".into(),
            client_id: "client123".into(),
            client_secret: None,
            code: None,
            username: None,
            password: None,
            refresh_token: None,
            redirect_uri: None,
        };
        let err = strategies.authorization_code(params).await.err().unwrap();
        assert!(matches!(err, CoreError::InternalServerError));
    }

    #[tokio::test]
    async fn authorization_code_session_not_found_errors() {
        let mut repos = build_repos_mock();
        let mut session_repo = get_mock_auth_session_repository_with_clone_expectations();
        session_repo
            .expect_get_by_code()
            .returning(move |_c: String| Box::pin(ready(Ok(None))));
        repos.auth_session_repository = AuthSessionRepoAny::Mock(session_repo);
        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id: Uuid::new_v4(),
            base_url: "https://auth.local".into(),
            realm_name: "master".into(),
            client_id: "client123".into(),
            client_secret: None,
            code: Some("abc".into()),
            username: None,
            password: None,
            refresh_token: None,
            redirect_uri: None,
        };
        let err = strategies.authorization_code(params).await.err().unwrap();
        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn password_secret_mismatch_when_required() {
        let mut repos = build_repos_mock();
        let mut client_repo = if let ClientRepoAny::Mock(m) = repos.client_repository { m } else { unreachable!() };
        let realm_id = Uuid::new_v4();
        let client = Client::new(ClientConfig {
            realm_id,
            name: "app".to_string(),
            client_id: "client123".to_string(),
            secret: Some("right".to_string()),
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: false,
            client_type: "confidential".to_string(),
            direct_access_grants_enabled: Some(false),
        });
        let client_clone = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |_cid: String, _rid: Uuid| Box::pin(ready(Ok(client_clone.clone()))));
        repos.client_repository = ClientRepoAny::Mock(client_repo);
        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id,
            base_url: "https://auth.local".to_string(),
            realm_name: "master".to_string(),
            client_id: client.client_id.clone(),
            client_secret: Some("wrong".to_string()),
            code: None,
            username: Some("alice".to_string()),
            password: Some("password".to_string()),
            refresh_token: None,
            redirect_uri: None,
        };
        let err = strategies.password(params).await.err().unwrap();
        assert!(matches!(err, CoreError::InvalidClientSecret));
    }

    #[tokio::test]
    async fn password_wrong_password_errors_invalid() {
        // Arrange
        let mut repos = build_repos_mock();
        let realm_id = Uuid::new_v4();
        let mut client_repo = if let ClientRepoAny::Mock(m) = repos.client_repository { m } else { unreachable!() };
        let mut user_repo = if let UserRepoAny::Mock(m) = repos.user_repository { m } else { unreachable!() };
        let mut cred_repo = get_mock_credential_repository_with_clone_expectations();

        let client = Client::new(ClientConfig {
            realm_id,
            name: "app".to_string(),
            client_id: "client123".to_string(),
            secret: None,
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: true,
            service_account_enabled: false,
            client_type: "public".to_string(),
            direct_access_grants_enabled: Some(true),
        });
        let user = User::new(UserConfig {
            realm_id,
            client_id: None,
            username: "alice".to_string(),
            firstname: "Alice".to_string(),
            lastname: "Doe".to_string(),
            email: "alice@example.com".to_string(),
            email_verified: true,
            enabled: true,
        });

        let hasher = crate::infrastructure::repositories::argon2_hasher::Argon2HasherRepository::new();
        let hash = hasher.hash_password("correct").await.unwrap();

        let credential = Credential::new(CredentialConfig {
            id: Uuid::new_v4(),
            salt: Some(hash.salt.clone()),
            credential_type: "password".into(),
            user_id: user.id,
            user_label: None,
            secret_data: hash.hash.clone(),
            credential_data: CredentialData::new(hash.credential_data.hash_iterations, hash.credential_data.algorithm.clone()),
            temporary: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let client_for_mock = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |_cid: String, _rid: Uuid| Box::pin(ready(Ok(client_for_mock.clone()))));
        user_repo
            .expect_get_by_username()
            .returning(move |_u: String, _rid: Uuid| Box::pin(ready(Ok(user.clone()))));
        cred_repo
            .expect_get_password_credential()
            .returning(move |_uid: Uuid| Box::pin(ready(Ok(credential.clone()))));

        repos.client_repository = ClientRepoAny::Mock(client_repo);
        repos.user_repository = UserRepoAny::Mock(user_repo);
        repos.credential_repository = crate::infrastructure::credential::CredentialRepoAny::Mock(cred_repo);

        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id,
            base_url: "https://auth.local".to_string(),
            realm_name: "master".to_string(),
            client_id: client.client_id.clone(),
            client_secret: None,
            code: None,
            username: Some("alice".to_string()),
            password: Some("wrong".to_string()),
            refresh_token: None,
            redirect_uri: None,
        };
        let err = strategies.password(params).await.err().unwrap();
        assert!(matches!(err, CoreError::Invalid));
    }

    #[tokio::test]
    async fn refresh_token_success_deletes_old_and_returns_new_tokens() {
        let mut repos = build_repos_mock();
        let mut user_repo = if let UserRepoAny::Mock(m) = repos.user_repository { m } else { unreachable!() };
        let mut ks = get_mock_key_store_repository_with_clone_expectations();
        let mut rr = get_mock_refresh_token_repository_with_clone_expectations();
        let realm_id = Uuid::new_v4();
        let user = User::new(UserConfig {
            realm_id,
            client_id: None,
            username: "bob".into(),
            firstname: "Bob".into(),
            lastname: "Smith".into(),
            email: "bob@example.com".into(),
            email_verified: true,
            enabled: true,
        });
        let user_for_mock = user.clone();
        let user_id_for_claims = user.id;
        user_repo
            .expect_get_by_id()
            .returning(move |uid: Uuid| {
                assert_eq!(uid, user_id_for_claims);
                Box::pin(ready(Ok(user_for_mock.clone())))
            });

        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm_id, Uuid::new_v4()).unwrap();
        let kp_for_mock = keypair.clone();
        ks.expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(kp_for_mock.clone()))));

        // Original refresh token claims
        let mut claims = JwtClaim {
            sub: user.id,
            iat: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4(),
            iss: "iss".into(),
            typ: ClaimsTyp::Refresh,
            azp: "client123".into(),
            aud: vec!["aud".into()],
            exp: Some(chrono::Utc::now().timestamp() + 3600),
            preferred_username: None,
            email: None,
            client_id: None,
        };
        let old_jti = claims.jti;
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256), &claims, &keypair.encoding_key).unwrap();

        rr.expect_get_by_jti()
            .returning(move |jti: Uuid| {
                assert_eq!(jti, old_jti);
                let rt = RefreshToken::new(Uuid::new_v4(), jti, user.id, false, Some(chrono::Utc::now() + chrono::Duration::hours(1)), chrono::Utc::now());
                Box::pin(ready(Ok(rt)))
            });
        // New refresh token creation assertion happens inside create_jwt; just allow it
        rr.expect_create()
            .returning(move |_jti: Uuid, _uid: Uuid, _exp| {
                let rt = RefreshToken::new(Uuid::new_v4(), Uuid::new_v4(), user.id, false, None, chrono::Utc::now());
                Box::pin(ready(Ok(rt)))
            });
        // Old refresh token must be deleted
        let mut deleted = false;
        rr.expect_delete()
            .returning(move |jti: Uuid| {
                assert_eq!(jti, old_jti);
                // mark deleted
                deleted = true;
                Box::pin(ready(Ok(())))
            });

        repos.user_repository = UserRepoAny::Mock(user_repo);
        repos.keystore_repository = KeyStoreRepoAny::Mock(ks);
        repos.refresh_token_repository = RefreshTokenRepoAny::Mock(rr);

        let strategies = build_strategies(repos);
        let params = GrantTypeParams {
            realm_id,
            base_url: "https://auth.local".into(),
            realm_name: "master".into(),
            client_id: "client123".into(),
            client_secret: None,
            code: None,
            username: None,
            password: None,
            refresh_token: Some(token),
            redirect_uri: None,
        };

        let _out = strategies.refresh_token(params).await.unwrap();
    }

    #[tokio::test]
    async fn verify_token_with_bad_signature_returns_validation_error() {
        let mut repos = build_repos_mock();
        let realm_id = Uuid::new_v4();

        // Keystore returns keypair A
        let mut ks = get_mock_key_store_repository_with_clone_expectations();
        let (priv_a, pub_a) = JwtKeyPair::generate().unwrap();
        let kp_a = JwtKeyPair::from_pem(&priv_a, &pub_a, realm_id, Uuid::new_v4()).unwrap();
        let kp_a_for_mock = kp_a.clone();
        ks.expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(kp_a_for_mock.clone()))));
        repos.keystore_repository = KeyStoreRepoAny::Mock(ks);

        // Sign token with a different keypair B
        let (priv_b, pub_b) = JwtKeyPair::generate().unwrap();
        let kp_b = JwtKeyPair::from_pem(&priv_b, &pub_b, realm_id, Uuid::new_v4()).unwrap();
        let mut claims = JwtClaim::new(Uuid::new_v4(), "u".into(), "iss".into(), vec!["aud".into()], ClaimsTyp::Bearer, "azp".into(), None);
        claims.exp = Some(chrono::Utc::now().timestamp() + 3600);
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256), &claims, &kp_b.encoding_key).unwrap();

        let strategies = build_strategies(repos);
        let err = strategies.verify_token(token, realm_id).await.err().unwrap();
        match err {
            CoreError::TokenValidationError(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
