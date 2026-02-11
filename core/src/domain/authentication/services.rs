use std::sync::Arc;

use chrono::{TimeZone, Utc};
use ferriskey_security::jwt::ports::KeyStoreRepository;
use jsonwebtoken::{Header, Validation};
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::domain::{
    abyss::federation::ports::FederationRepository,
    authentication::{
        ScopeManager,
        entities::{
            AuthInput, AuthOutput, AuthSession, AuthSessionParams, AuthenticateOutput,
            AuthenticationMethod, AuthenticationStepStatus, AuthorizeRequestInput,
            AuthorizeRequestOutput, CredentialsAuthParams, ExchangeTokenInput, GrantType, JwtToken,
            TokenIntrospectionResponse,
        },
        ports::{AuthService, AuthSessionRepository},
        value_objects::{
            AuthenticationResult, GenerateTokenInput, GetUserInfoInput, GrantTypeParams, Identity,
            IntrospectTokenInput, RegisterUserInput, UserInfoResponse,
        },
    },
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, generate_random_string},
    credential::{entities::CredentialData, ports::CredentialRepository},
    crypto::HasherRepository,
    jwt::{
        entities::{ClaimsTyp, IdTokenClaims, JwkKey, Jwt, JwtClaim, TokenClaims},
        ports::{AccessTokenRepository, RefreshTokenRepository},
    },
    realm::{entities::RealmId, ports::RealmRepository},
    role::ports::RoleRepository,
    user::{
        entities::RequiredAction,
        ports::{UserRepository, UserRoleRepository},
        value_objects::CreateUserRequest,
    },
};
use crate::infrastructure::abyss::federation::ldap::LdapClientImpl;

#[derive(Clone, Debug)]
pub struct AuthServiceImpl<R, C, RU, U, URR, RR, CR, H, AS, KS, RT, AT, F>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    U: UserRepository,
    URR: UserRoleRepository,
    RR: RoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) redirect_uri_repository: Arc<RU>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) user_role_repository: Arc<URR>,
    pub(crate) role_repository: Arc<RR>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) auth_session_repository: Arc<AS>,
    pub(crate) keystore_repository: Arc<KS>,
    pub(crate) refresh_token_repository: Arc<RT>,
    pub(crate) access_token_repository: Arc<AT>,
    pub(crate) federation_repository: Arc<F>,
    pub(crate) ldap_client: LdapClientImpl,
}

impl<R, C, RU, U, URR, RR, CR, H, AS, KS, RT, AT, F>
    AuthServiceImpl<R, C, RU, U, URR, RR, CR, H, AS, KS, RT, AT, F>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    U: UserRepository,
    URR: UserRoleRepository,
    RR: RoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        redirect_uri_repository: Arc<RU>,
        user_repository: Arc<U>,
        user_role_repository: Arc<URR>,
        role_repository: Arc<RR>,
        credential_repository: Arc<CR>,
        hasher_repository: Arc<H>,
        auth_session_repository: Arc<AS>,
        keystore_repository: Arc<KS>,
        refresh_token_repository: Arc<RT>,
        access_token_repository: Arc<AT>,
        federation_repository: Arc<F>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            redirect_uri_repository,
            user_repository,
            user_role_repository,
            role_repository,
            credential_repository,
            hasher_repository,
            auth_session_repository,
            keystore_repository,
            refresh_token_repository,
            access_token_repository,
            federation_repository,
            ldap_client: LdapClientImpl,
        }
    }
}

impl<R, C, RU, U, URR, RR, CR, H, AS, KS, RT, AT, F>
    AuthServiceImpl<R, C, RU, U, URR, RR, CR, H, AS, KS, RT, AT, F>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    U: UserRepository,
    URR: UserRoleRepository,
    RR: RoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
{
    fn expires_in_from(exp: i64) -> u32 {
        let now = Utc::now().timestamp();
        if exp <= now { 0 } else { (exp - now) as u32 }
    }

    async fn generate_token(&self, claims: JwtClaim, realm_id: RealmId) -> Result<Jwt, CoreError> {
        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(jwt_key_pair.id.to_string());
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

    async fn try_generate_token<T>(&self, claims: &T, realm_id: RealmId) -> Result<Jwt, CoreError>
    where
        T: TokenClaims,
    {
        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(jwt_key_pair.id.to_string());
        let token =
            jsonwebtoken::encode(&header, &claims, &jwt_key_pair.encoding_key).map_err(|e| {
                tracing::error!("JWT generation error: {}", e);

                CoreError::TokenGenerationError(e.to_string())
            })?;

        Ok(Jwt {
            token,
            expires_at: claims.get_exp(),
        })
    }

    async fn create_jwt(
        &self,
        input: GenerateTokenInput,
    ) -> Result<(Jwt, Jwt, Option<Jwt>), CoreError> {
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
            input.scope.clone(),
        );

        let jwt = self.generate_token(claims.clone(), input.realm_id).await?;

        // Persist access tokens so backend services can introspect/revoke them immediately.
        let access_token_hash = format!("{:x}", Sha256::digest(jwt.token.as_bytes()));
        let access_token_claims =
            serde_json::to_value(&claims).map_err(|_| CoreError::InternalServerError)?;
        let access_token_expires_at = claims
            .exp
            .and_then(|exp| Utc.timestamp_opt(exp, 0).single());

        self.access_token_repository
            .create(
                access_token_hash,
                Some(claims.jti),
                claims.sub,
                input.realm_id,
                access_token_expires_at,
                access_token_claims,
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let refresh_claims = JwtClaim::new_refresh_token(
            claims.sub,
            claims.iss.clone(),
            claims.aud.clone(),
            claims.azp,
            claims.scope.clone(),
        );

        let refresh_token = self
            .generate_token(refresh_claims.clone(), input.realm_id)
            .await?;

        let contains_openid_scope = input.scope.as_ref().is_some_and(|s| s.contains("openid"));
        let exp = claims.exp.unwrap_or(0);
        let iat = Utc::now().timestamp();
        let preferred_username: String = claims.preferred_username.clone().unwrap_or_default();

        let id_token: Option<Jwt> = if contains_openid_scope {
            let aud = claims.aud.join(" ");
            let id_claims = IdTokenClaims {
                iss: claims.iss,
                aud,
                auth_time: None,
                email: claims.email.clone(),
                email_verified: None,
                exp,
                iat,
                preferred_username,
                sub: claims.sub,
            };
            let t = self.try_generate_token(&id_claims, input.realm_id).await?;

            Some(t)
        } else {
            None
        };

        self.refresh_token_repository
            .create(
                refresh_claims.jti,
                input.user_id,
                Some(Utc.timestamp_opt(refresh_token.expires_at, 0).unwrap()),
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok((jwt, refresh_token, id_token))
    }

    async fn verify_token(&self, token: String, realm_id: RealmId) -> Result<JwtClaim, CoreError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);

        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        validation.validate_aud = false;
        let token_data =
            jsonwebtoken::decode::<JwtClaim>(&token, &jwt_key_pair.decoding_key, &validation)
                .map_err(|e| CoreError::TokenValidationError(e.to_string()))?;

        let current_time = Utc::now().timestamp();

        if let Some(exp) = token_data.claims.exp
            && exp < current_time
        {
            return Err(CoreError::ExpiredToken);
        }

        Ok(token_data.claims)
    }

    async fn verify_password(&self, user_id: Uuid, password: String) -> Result<bool, CoreError> {
        let credential = self
            .credential_repository
            .get_password_credential(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let salt = credential.salt.ok_or(CoreError::InternalServerError)?;

        let CredentialData::Hash {
            hash_iterations,
            algorithm,
        } = credential.credential_data
        else {
            return Err(CoreError::InternalServerError);
        };

        let is_valid = self
            .hasher_repository
            .verify_password(
                &password,
                &credential.secret_data,
                hash_iterations,
                &algorithm,
                &salt,
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(is_valid)
    }

    async fn verify_refresh_token(
        &self,
        token: String,
        realm_id: RealmId,
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

    async fn authorization_code(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let code = params.code.ok_or(CoreError::InternalServerError)?;

        let auth_session = self
            .auth_session_repository
            .get_by_code(code)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::NotFound)?;

        let user_id = auth_session.user_id.ok_or(CoreError::NotFound)?;
        let user = self.user_repository.get_by_id(user_id).await?;

        let scope_manager = ScopeManager::new();
        let final_scope = scope_manager.allowed_scopes();

        let (jwt, refresh_token, id_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
                scope: Some(final_scope),
            })
            .await?;

        let id_token_value = id_token.map(|t| t.token);

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            id_token_value,
        ))
    }

    async fn client_credential(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if client.secret != params.client_secret {
            return Err(CoreError::InvalidClientSecret);
        }

        info!("try to fetch user client, client id: {}", client.id);

        let user = self
            .user_repository
            .get_by_client_id(client.id)
            .await
            .map_err(|e| match e {
                CoreError::NotFound => CoreError::ServiceAccountNotFound,
                _ => CoreError::InternalServerError,
            })?;

        let scope_manager = ScopeManager::new();
        let final_scope = scope_manager.merge_with_defaults(params.scope);

        let (jwt, refresh_token, id_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
                scope: Some(final_scope),
            })
            .await?;

        let id_token_value = id_token.map(|t| t.token);

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            id_token_value,
        ))
    }

    async fn password(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let username = params.username.ok_or(CoreError::InternalServerError)?;
        let password = params.password.ok_or(CoreError::InternalServerError)?;

        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if !client.public_client {
            if !client.direct_access_grants_enabled && params.client_secret.is_none() {
                return Err(CoreError::InvalidClientSecret);
            }

            if let Some(provided_secret) = params.client_secret
                && client.secret != Some(provided_secret)
            {
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

        let scope_manager = ScopeManager::new();
        let final_scope = scope_manager.merge_with_defaults(params.scope);

        let (jwt, refresh_token, id_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
                scope: Some(final_scope),
            })
            .await?;

        let id_token_value = id_token.map(|t| t.token);

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            id_token_value,
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

        let (jwt, refresh_token, id_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                email: user.email,
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
                scope: claims.scope.clone(),
            })
            .await?;

        self.refresh_token_repository
            .delete(claims.jti)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let id_token_value = id_token.map(|t| t.token);

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            id_token_value,
        ))
    }

    async fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> Result<JwtToken, CoreError> {
        match grant_type {
            GrantType::Code => self.authorization_code(params).await,
            GrantType::Password => self.password(params).await,
            GrantType::Credentials => self.client_credential(params).await,
            GrantType::RefreshToken => self.refresh_token(params).await,
        }
    }

    async fn handle_user_credentials_authentication(
        &self,
        params: CredentialsAuthParams,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
        let auth_result = self
            .using_session_code(
                params.realm_name,
                params.client_id,
                params.session_code,
                params.username,
                params.password,
                params.base_url,
            )
            .await
            .map_err(|e| {
                warn!("authentication using session code error: {:?}", e);
                e
            })?;

        self.determine_next_step(auth_result, params.session_code, auth_session)
            .await
    }

    async fn determine_next_step(
        &self,
        auth_result: AuthenticationResult,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
        if !auth_result.required_actions.is_empty() {
            return Ok(AuthenticateOutput::requires_actions(
                auth_result.user_id,
                auth_result.required_actions,
                auth_result.token.ok_or(CoreError::InternalServerError)?,
            ));
        }

        let has_otp_credentials = auth_result.credentials.iter().any(|cred| cred == "otp");
        let needs_configure_otp = auth_result
            .required_actions
            .contains(&RequiredAction::ConfigureOtp);

        if has_otp_credentials && !needs_configure_otp {
            let token = auth_result.token.ok_or(CoreError::InternalServerError)?;
            return Ok(AuthenticateOutput::requires_otp_challenge(
                auth_result.user_id,
                token,
            ));
        }

        self.finalize_authentication(auth_result.user_id, session_code, auth_session)
            .await
    }

    async fn finalize_authentication(
        &self,
        user_id: Uuid,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
        let authorization_code = generate_random_string();

        self.auth_session_repository
            .update_code_and_user_id(session_code, authorization_code.clone(), user_id)
            .await
            .map_err(|e| {
                warn!(
                    "failed to update auth session with code and user id: {:?}",
                    e
                );
                CoreError::InternalServerError
            })?;

        let redirect_uri = self.build_redirect_url(&auth_session, &authorization_code)?;

        Ok(AuthenticateOutput::complete_with_redirect(
            user_id,
            authorization_code,
            redirect_uri,
        ))
    }

    async fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> Result<AuthenticationResult, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.client_repository
            .get_by_client_id(client_id.clone(), realm.id)
            .await
            .map_err(|e| {
                warn!("Client not found for client_id {}: {:?}", client_id, e);

                CoreError::InvalidClient
            })?;

        let user = self
            .user_repository
            .get_by_username(username.clone(), realm.id)
            .await
            .map_err(|e| {
                warn!("User not found for username {}: {:?}", username, e);

                CoreError::UserNotFound
            })?;

        // Check if user has federation mapping (LDAP authentication) FIRST
        let federation_mapping = self
            .federation_repository
            .get_mapping_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        info!(
            "User {} (ID: {}): federation_mapping = {}",
            user.username,
            user.id,
            if federation_mapping.is_some() {
                "YES (LDAP user)"
            } else {
                "NO (local user)"
            }
        );

        let (has_valid_password, credentials, has_temporary_password) =
            if let Some(mapping) = federation_mapping {
                // User is federated - authenticate via LDAP
                info!(
                    "User {} is federated (provider_id: {}), authenticating via LDAP",
                    user.username, mapping.provider_id
                );

                let provider = self
                    .federation_repository
                    .get_by_id(mapping.provider_id)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?
                    .ok_or(CoreError::InternalServerError)?;

                if !provider.enabled {
                    error!("Federation provider {} is disabled", provider.name);
                    return Err(CoreError::InvalidPassword);
                }

                // Authenticate via LDAP
                let ldap_auth_result = match self
                    .ldap_client
                    .authenticate_user(&provider, &user.username, &password)
                    .await
                {
                    Ok(_) => {
                        info!("LDAP authentication successful for user {}", user.username);
                        true
                    }
                    Err(e) => {
                        error!(
                            "LDAP authentication failed for user {}: {}",
                            user.username, e
                        );
                        false
                    }
                };

                // Federated users don't have local credentials
                (ldap_auth_result, vec!["federated".to_string()], false)
            } else {
                // User is not federated - use local password hash
                info!(
                    "User {} is not federated, using local password hash",
                    user.username
                );

                let user_credentials = self
                    .credential_repository
                    .get_credentials_by_user_id(user.id)
                    .await
                    .map_err(|_| CoreError::GetUserCredentialsError)?;

                let has_temp_password = user_credentials.iter().any(|cred| cred.temporary);

                let creds: Vec<String> = user_credentials
                    .iter()
                    .map(|cred| cred.credential_type.clone().to_string())
                    .collect();

                let credential = self
                    .credential_repository
                    .get_password_credential(user.id)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;

                let salt = credential.salt.ok_or(CoreError::InternalServerError)?;

                let CredentialData::Hash {
                    hash_iterations,
                    algorithm,
                } = &credential.credential_data
                else {
                    tracing::error!(
                        "A password credential doesn't have Hash credential data.
This is a server error that should be investigated. Do not forward back this message to the client"
                    );
                    return Err(CoreError::InternalServerError);
                };

                let is_valid = self
                    .hasher_repository
                    .verify_password(
                        &password,
                        &credential.secret_data,
                        *hash_iterations,
                        algorithm,
                        &salt,
                    )
                    .await
                    .map_err(|_| CoreError::InvalidPassword)?;

                (is_valid, creds, has_temp_password)
            };

        if !has_valid_password {
            return Err(CoreError::InvalidPassword);
        }

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let iss = format!("{}/realms/{}", base_url, realm.name);

        let jwt_claim = JwtClaim::new(
            user.id,
            user.username.clone(),
            iss,
            vec![format!("{}-realm", realm.name), "account".to_string()],
            ClaimsTyp::Bearer,
            client_id.clone(),
            Some(user.email.clone()),
            Some(auth_session.scope),
        );

        if !user.required_actions.is_empty() || has_temporary_password {
            let jwt_token = self.generate_token(jwt_claim, realm.id).await?;

            let required_actions = if has_temporary_password {
                vec![RequiredAction::UpdatePassword]
            } else {
                user.required_actions.clone()
            };

            return Ok(AuthenticationResult {
                code: None,
                required_actions,
                user_id: user.id,
                token: Some(jwt_token.token),
                credentials,
            });
        }
        let has_otp_credentials = credentials.iter().any(|cred| cred == "otp");
        if has_otp_credentials {
            let jwt_token = self.generate_token(jwt_claim, realm.id).await?;

            return Ok(AuthenticationResult {
                code: None,
                required_actions: user.required_actions.clone(),
                user_id: user.id,
                token: Some(jwt_token.token),
                credentials,
            });
        }

        Ok(AuthenticationResult {
            code: Some(generate_random_string()),
            required_actions: Vec::new(),
            user_id: user.id,
            token: None,
            credentials,
        })
    }

    async fn handle_token_refresh(
        &self,
        token: String,
        realm_id: RealmId,
        auth_session: AuthSession,
        session_code: Uuid,
    ) -> Result<AuthenticateOutput, CoreError> {
        let claims = self
            .verify_token(token.clone(), realm_id)
            .await
            .map_err(|e| {
                error!("Failed to verify token: {:?}", e);
                e
            })?;

        let user = self
            .user_repository
            .get_by_id(claims.sub)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if !user.required_actions.is_empty() {
            let jwt_token = self.generate_token(claims, realm_id).await?;

            return Ok(AuthenticateOutput {
                status: AuthenticationStepStatus::RequiresActions,
                user_id: user.id,
                authorization_code: None,
                redirect_url: None,
                required_actions: user.required_actions,
                session_state: None,
                temporary_token: Some(jwt_token.token),
            });
        }

        self.finalize_authentication(claims.sub, session_code, auth_session)
            .await
    }

    fn build_redirect_url(
        &self,
        auth_session: &AuthSession,
        authorization_code: &str,
    ) -> Result<String, CoreError> {
        let state = auth_session
            .state
            .as_ref()
            .ok_or(CoreError::InternalServerError)?;

        Ok(format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, authorization_code, state
        ))
    }
}

impl<R, C, RU, U, URR, RR, CR, H, AS, KS, RT, AT, F> AuthService
    for AuthServiceImpl<R, C, RU, U, URR, RR, CR, H, AS, KS, RT, AT, F>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    U: UserRepository,
    URR: UserRoleRepository,
    RR: RoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
{
    async fn auth(&self, input: AuthInput) -> Result<AuthOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await?;

        let redirect_uri = input.redirect_uri.clone();

        let client_redirect_uris = self
            .redirect_uri_repository
            .get_enabled_by_client_id(client.id)
            .await?;

        if !client_redirect_uris.iter().any(|uri| {
            if uri.value == redirect_uri {
                return true;
            }

            if let Ok(regex) = regex::Regex::new(&uri.value) {
                return regex.is_match(&redirect_uri);
            }

            false
        }) {
            return Err(CoreError::InvalidClient);
        }

        if !client.enabled {
            return Err(CoreError::InvalidClient);
        }

        let params = AuthSessionParams {
            realm_id: realm.id,
            client_id: client.id,
            redirect_uri,
            response_type: input.response_type,
            scope: input.scope.unwrap_or_default(),
            state: input.state.clone(),
            nonce: None,
            user_id: None,
            code: None,
            authenticated: false,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
        };
        let session = self
            .auth_session_repository
            .create(&AuthSession::new(params))
            .await
            .map_err(|_| CoreError::SessionCreateError)?;

        let login_url = format!(
            "?client_id={}&redirect_uri={}&state={}",
            client.client_id,
            input.redirect_uri,
            input.state.unwrap_or_default()
        );

        Ok(AuthOutput { login_url, session })
    }

    async fn get_certs(&self, realm_name: String) -> Result<Vec<JwkKey>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let jwk_keypair = self
            .keystore_repository
            .get_or_generate_key(realm.id)
            .await
            .map_err(|_| CoreError::RealmKeyNotFound)?;

        let jwk_key = jwk_keypair
            .to_jwk_key()
            .map_err(|e| CoreError::InvalidKey(e.to_string()))?;

        Ok(vec![jwk_key])
    }

    async fn exchange_token(&self, input: ExchangeTokenInput) -> Result<JwtToken, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await?;

        let params = GrantTypeParams {
            realm_id: realm.id,
            base_url: input.base_url,
            realm_name: realm.name,
            client_id: input.client_id,
            client_secret: input.client_secret,
            code: input.code,
            username: input.username,
            password: input.password,
            refresh_token: input.refresh_token,
            redirect_uri: None,
            scope: input.scope,
        };

        self.authenticate_with_grant_type(input.grant_type, params)
            .await
    }

    async fn authorize_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> Result<AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Bearer {
            return Err(CoreError::InvalidToken);
        }

        let user = self.user_repository.get_by_id(input.claims.sub).await?;

        self.verify_token(input.token, user.realm_id).await?;

        let identity: Identity = match input.claims.is_service_account() {
            true => {
                let client_id = input.claims.client_id.ok_or(CoreError::InvalidClient)?;
                let client_id = Uuid::parse_str(&client_id).map_err(|e| {
                    tracing::error!("failed to parse client id: {:?}", e);
                    CoreError::InvalidClient
                })?;

                let client = self.client_repository.get_by_id(client_id).await?;

                Identity::Client(client)
            }
            false => Identity::User(user),
        };

        Ok(AuthorizeRequestOutput { identity })
    }

    async fn authenticate(
        &self,
        input: super::entities::AuthenticateInput,
    ) -> Result<super::entities::AuthenticateOutput, CoreError> {
        let auth_session = self
            .auth_session_repository
            .get_by_session_code(input.session_code)
            .await
            .map_err(|e| {
                warn!("Failed to get auth session by session code: {:?}", e);
                CoreError::InternalServerError
            })?;

        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        match input.auth_method {
            AuthenticationMethod::ExistingToken { token } => {
                self.handle_token_refresh(token, realm.id, auth_session, input.session_code)
                    .await
            }
            AuthenticationMethod::UserCredentials { username, password } => {
                let params = CredentialsAuthParams {
                    realm_name: input.realm_name,
                    client_id: input.client_id,
                    session_code: input.session_code,
                    base_url: input.base_url,
                    username,
                    password,
                };

                self.handle_user_credentials_authentication(params, auth_session)
                    .await
            }
        }
    }

    async fn register_user(
        &self,
        url: String,
        input: RegisterUserInput,
    ) -> Result<JwtToken, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let firstname: String = input.first_name.unwrap_or_else(|| "FirstName".to_string());
        let lastname: String = input.last_name.unwrap_or_else(|| "LastName".to_string());

        let user = self
            .user_repository
            .create_user(CreateUserRequest {
                client_id: None,
                email: input.email,
                email_verified: true,
                enabled: true,
                firstname,
                lastname,
                realm_id: realm.id,
                username: input.username,
            })
            .await?;

        // create user credentials
        let hash_result = self
            .hasher_repository
            .hash_password(&input.password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(user.id, "password".into(), hash_result, "".into(), false)
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        let iss = format!("{}/realms/{}", url, realm.name);
        let claims = JwtClaim::new(
            user.id,
            user.username.clone(),
            iss.clone(),
            vec![format!("{}-realm", realm.name), "account".to_string()],
            ClaimsTyp::Bearer,
            "".to_string(),
            Some(user.email.clone()),
            None,
        );

        let jwt = self.generate_token(claims.clone(), realm.id).await?;

        let refresh_claims =
            JwtClaim::new_refresh_token(claims.sub, claims.iss, claims.aud, claims.azp, None);

        let refresh_token = self
            .generate_token(refresh_claims.clone(), realm.id)
            .await?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            None,
        ))
    }

    async fn get_userinfo(
        &self,
        identity: Identity,
        input: GetUserInfoInput,
    ) -> Result<UserInfoResponse, CoreError> {
        let user = self.user_repository.get_by_id(identity.id()).await?;

        let scopes = input
            .claims
            .scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect::<Vec<_>>())
            .unwrap_or_default();

        let contains_openid = scopes.contains(&"openid".to_string());
        if scopes.is_empty() || !contains_openid {
            return Err(CoreError::InvalidToken);
        }

        let mut response = UserInfoResponse {
            sub: user.id.to_string(),
            ..Default::default()
        };

        if scopes.contains(&"profile".to_string()) {
            response.name = Some(format!("{} {}", user.firstname, user.lastname));
            response.given_name = Some(user.firstname.clone());
            response.family_name = Some(user.lastname.clone());
            response.preferred_username = Some(user.username.clone());
        }

        if scopes.contains(&"email".to_string()) {
            response.email = Some(user.email.clone());
            response.email_verified = Some(user.email_verified);
        }

        Ok(response)
    }

    async fn introspect_token(
        &self,
        input: IntrospectTokenInput,
    ) -> Result<TokenIntrospectionResponse, CoreError> {
        let inactive = TokenIntrospectionResponse {
            active: false,
            ..Default::default()
        };

        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if !client.enabled || client.public_client {
            return Err(CoreError::InvalidClient);
        }

        if client.secret.as_deref() != Some(&input.client_secret) {
            return Err(CoreError::InvalidClientSecret);
        }

        // Authorization: require service account to have the "introspect" role.
        if !client.service_account_enabled {
            return Err(CoreError::Forbidden(
                "Client service account is not enabled".to_string(),
            ));
        }

        let required_role = self
            .role_repository
            .find_by_name("introspect".to_string(), realm.id.into())
            .await?
            .ok_or(CoreError::Forbidden(
                "Missing required scope: introspect".to_string(),
            ))?;

        let service_account_user = self
            .user_repository
            .get_by_client_id(client.id)
            .await
            .map_err(|_| CoreError::ServiceAccountNotFound)?;

        let roles = self
            .user_role_repository
            .get_user_roles(service_account_user.id)
            .await?;

        let has_introspect = roles.iter().any(|r| r.id == required_role.id);
        if !has_introspect {
            return Err(CoreError::Forbidden(
                "Missing required scope: introspect".to_string(),
            ));
        }

        let token = input.token;
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

        // Opaque token support: prefer DB lookup by hash. This also enables immediate revocation.
        if let Some(stored) = self
            .access_token_repository
            .get_by_token_hash(token_hash.clone())
            .await
            .map_err(|_| CoreError::InternalServerError)?
        {
            if stored.revoked {
                return Ok(inactive);
            }

            if let Some(expires_at) = stored.expires_at
                && expires_at < Utc::now()
            {
                return Ok(inactive);
            }

            let claims: JwtClaim = serde_json::from_value(stored.claims)
                .map_err(|_| CoreError::InternalServerError)?;

            return Ok(TokenIntrospectionResponse {
                active: true,
                scope: claims.scope,
                client_id: Some(claims.azp),
                username: claims.preferred_username,
                sub: Some(claims.sub.to_string()),
                token_type: Some(match claims.typ {
                    ClaimsTyp::Bearer => "Bearer".to_string(),
                    ClaimsTyp::Refresh => "Refresh".to_string(),
                    ClaimsTyp::Temporary => "Temporary".to_string(),
                }),
                exp: claims.exp,
                iat: Some(claims.iat),
                nbf: Some(claims.iat),
                aud: Some(claims.aud.join(" ")),
                iss: Some(claims.iss),
                jti: Some(claims.jti.to_string()),
                realm: Some(realm.name),
            });
        }

        // Backward-compatible JWT introspection: validate signature + expiry even if not persisted.
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Ok(inactive);
        }

        let mut claims = match self.verify_token(token.clone(), realm.id).await {
            Ok(c) => c,
            Err(_) => return Ok(inactive),
        };

        // If the token is a refresh token (or hinted as such), enforce refresh token repository checks.
        if input.token_type_hint.as_deref() == Some("refresh_token")
            || claims.typ == ClaimsTyp::Refresh
        {
            claims = match self.verify_refresh_token(token, realm.id).await {
                Ok(c) => c,
                Err(_) => return Ok(inactive),
            };
        }

        Ok(TokenIntrospectionResponse {
            active: true,
            scope: claims.scope,
            client_id: Some(claims.azp),
            username: claims.preferred_username,
            sub: Some(claims.sub.to_string()),
            token_type: Some(match claims.typ {
                ClaimsTyp::Bearer => "Bearer".to_string(),
                ClaimsTyp::Refresh => "Refresh".to_string(),
                ClaimsTyp::Temporary => "Temporary".to_string(),
            }),
            exp: claims.exp,
            iat: Some(claims.iat),
            nbf: Some(claims.iat),
            aud: Some(claims.aud.join(" ")),
            iss: Some(claims.iss),
            jti: Some(claims.jti.to_string()),
            realm: Some(realm.name),
        })
    }
}
