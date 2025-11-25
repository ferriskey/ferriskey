use std::sync::Arc;

use chrono::{TimeZone, Utc};
use jsonwebtoken::{Header, Validation};
use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{
            AuthInput, AuthOutput, AuthSession, AuthSessionParams, AuthenticateOutput,
            AuthenticationMethod, AuthenticationStepStatus, AuthorizeRequestOutput,
            CredentialsAuthParams, ExchangeTokenInput, GrantType, JwtToken,
        },
        ports::{AuthService, AuthSessionRepository, AuthenticatePort, GrantTypeService},
        services::grant_type_service::GenerateTokenInput,
        value_objects::{AuthenticationResult, GrantTypeParams, Identity, RegisterUserInput},
    },
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, generate_random_string, services::Service},
    credential::{entities::CredentialData, ports::CredentialRepository},
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::{
        entities::{ClaimsTyp, JwkKey, Jwt, JwtClaim},
        ports::{KeyStoreRepository, RefreshTokenRepository},
    },
    realm::{entities::RealmId, ports::RealmRepository},
    role::ports::RoleRepository,
    seawatch::SecurityEventRepository,
    trident::ports::RecoveryCodeRepository,
    user::{
        entities::RequiredAction,
        ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
        value_objects::CreateUserRequest,
    },
    webhook::ports::WebhookRepository,
};

pub mod authenticate;
pub mod grant_type_service;

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE> AuthService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE>
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
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
    SE: SecurityEventRepository,
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
        };

        self.authenticate_with_grant_type(input.grant_type, params)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn authorize_request(
        &self,
        input: super::entities::AuthorizeRequestInput,
    ) -> Result<super::entities::AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Bearer {
            return Err(CoreError::InternalServerError);
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
            .map_err(|_| CoreError::InternalServerError)?;

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
        );

        let jwt = self.generate_token(claims.clone(), realm.id).await?;

        let refresh_claims =
            JwtClaim::new_refresh_token(claims.sub, claims.iss, claims.aud, claims.azp);

        let refresh_token = self
            .generate_token(refresh_claims.clone(), realm.id)
            .await?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            jwt.expires_at as u32,
            "id_token".to_string(),
        ))
    }
}

pub struct AuthServiceImpl<R, C, RU, U, CR, H, AS, KS, RT>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) redirect_uri_repository: Arc<RU>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) auth_session_repository: Arc<AS>,
    pub(crate) keystore_repository: Arc<KS>,
    pub(crate) refresh_token_repository: Arc<RT>,
}

impl<R, C, RU, U, CR, H, AS, KS, RT> AuthServiceImpl<R, C, RU, U, CR, H, AS, KS, RT>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
{
    async fn generate_token(&self, claims: JwtClaim, realm_id: RealmId) -> Result<Jwt, CoreError> {
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
            .map_err(|_| CoreError::InvalidClient)?;

        if !client.direct_access_grants_enabled {
            if params.client_secret.is_none() {
                return Err(CoreError::InvalidClientSecret);
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
            .await?;

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
            .map_err(|_| CoreError::InternalServerError)?;

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
            .await?;

        let user = self
            .user_repository
            .get_by_username(username, realm.id)
            .await?;

        let user_credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        let has_temporary_password = user_credentials.iter().any(|cred| cred.temporary);

        let credentials: Vec<String> = user_credentials
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

        let has_valid_password = self
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

        if !has_valid_password {
            return Err(CoreError::InvalidPassword);
        }
        let iss = format!("{}/realms/{}", base_url, realm.name);

        let jwt_claim = JwtClaim::new(
            user.id,
            user.username.clone(),
            iss,
            vec![format!("{}-realm", realm.name), "account".to_string()],
            ClaimsTyp::Bearer,
            client_id.clone(),
            Some(user.email.clone()),
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

        self.auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

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
        let claims = self.verify_refresh_token(token.clone(), realm_id).await?;

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

impl<R, C, RU, U, CR, H, AS, KS, RT> AuthService for AuthServiceImpl<R, C, RU, U, CR, H, AS, KS, RT>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
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
        };

        self.authenticate_with_grant_type(input.grant_type, params)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn authorize_request(
        &self,
        input: super::entities::AuthorizeRequestInput,
    ) -> Result<super::entities::AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Bearer {
            return Err(CoreError::InternalServerError);
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
            .map_err(|_| CoreError::InternalServerError)?;

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
        );

        let jwt = self.generate_token(claims.clone(), realm.id).await?;

        let refresh_claims =
            JwtClaim::new_refresh_token(claims.sub, claims.iss, claims.aud, claims.azp);

        let refresh_token = self
            .generate_token(refresh_claims.clone(), realm.id)
            .await?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            jwt.expires_at as u32,
            "id_token".to_string(),
        ))
    }
}
