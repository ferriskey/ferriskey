use chrono::{TimeZone, Utc};
use jsonwebtoken::{Header, Validation};
use uuid::Uuid;

use crate::domain::{
    authentication::{
        ports::AuthSessionRepository, services::grant_type_service::GenerateTokenInput,
    },
    client::{
        ports::{ClientRepository, RedirectUriRepository},
        value_objects::CreateClientRequest,
    },
    common::{
        entities::{InitializationResult, StartupConfig, app_errors::CoreError},
        generate_random_string,
        policies::FerriskeyPolicy,
        ports::CoreService,
    },
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::{
        entities::{ClaimsTyp, Jwt, JwtClaim},
        ports::{KeyStoreRepository, RefreshTokenRepository},
    },
    realm::ports::RealmRepository,
    role::{
        entities::permission::Permissions, ports::RoleRepository, value_objects::CreateRoleRequest,
    },
    trident::ports::RecoveryCodeRepository,
    user::{
        ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
        value_objects::CreateUserRequest,
    },
    webhook::ports::{WebhookNotifierRepository, WebhookRepository},
};

#[derive(Clone)]
pub struct Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
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
    pub(crate) realm_repository: R,
    pub(crate) client_repository: C,
    pub(crate) user_repository: U,
    pub(crate) credential_repository: CR,
    pub(crate) hasher_repository: H,
    pub(crate) auth_session_repository: AS,
    pub(crate) redirect_uri_repository: RU,
    pub(crate) role_repository: RO,
    pub(crate) keystore_repository: KS,
    pub(crate) user_role_repository: UR,
    pub(crate) user_required_action_repository: URA,
    pub(crate) health_check_repository: HC,
    pub(crate) webhook_repository: W,
    pub(crate) webhook_notifier_repository: WN,
    pub(crate) refresh_token_repository: RT,
    pub(crate) recovery_code_repository: RC,

    pub(crate) policy: FerriskeyPolicy<U, C, UR>,
}

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
    Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: R,
        client_repository: C,
        user_repository: U,
        credential_repository: CR,
        hasher_repository: H,
        auth_session_repository: AS,
        redirect_uri_repository: RU,
        role_repository: RO,
        keystore_repository: KS,
        user_role_repository: UR,
        user_required_action_repository: URA,
        health_check_repository: HC,
        webhook_repository: W,
        webhook_notifier_repository: WN,
        refresh_token_repository: RT,
        recovery_code_repository: RC,
    ) -> Self {
        let policy = FerriskeyPolicy::new(
            user_repository.clone(),
            client_repository.clone(),
            user_role_repository.clone(),
        );

        Service {
            realm_repository,
            client_repository,
            user_repository,
            credential_repository,
            hasher_repository,
            auth_session_repository,
            redirect_uri_repository,
            role_repository,
            keystore_repository,
            user_role_repository,
            user_required_action_repository,
            health_check_repository,
            webhook_repository,
            webhook_notifier_repository,
            refresh_token_repository,
            recovery_code_repository,

            policy,
        }
    }

    pub(crate) async fn verify_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> Result<bool, CoreError> {
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

    pub(crate) async fn generate_token(
        &self,
        claims: JwtClaim,
        realm_id: Uuid,
    ) -> Result<Jwt, CoreError> {
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

    pub(crate) async fn create_jwt(
        &self,
        input: GenerateTokenInput,
    ) -> Result<(Jwt, Jwt), CoreError> {
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

    pub(crate) async fn verify_token(
        &self,
        token: String,
        realm_id: Uuid,
    ) -> Result<JwtClaim, CoreError> {
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

    pub(crate) async fn verify_refresh_token(
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

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC> CoreService
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
    async fn initialize_application(
        &self,
        config: StartupConfig,
    ) -> Result<InitializationResult, CoreError> {
        let realm = match self
            .realm_repository
            .get_by_name(config.master_realm_name.clone())
            .await
        {
            Ok(Some(realm)) => {
                tracing::info!("{} already exists", config.master_realm_name);
                realm
            }
            Ok(None) => {
                tracing::info!("creating master realm");

                let realm = self
                    .realm_repository
                    .create_realm(config.master_realm_name.clone())
                    .await?;

                tracing::info!("{} realm created", config.master_realm_name);
                realm
            }
            Err(_) => {
                tracing::info!("creating master realm");
                let realm = self
                    .realm_repository
                    .create_realm(config.master_realm_name.clone())
                    .await?;

                tracing::info!("{} realm created", config.master_realm_name);
                realm
            }
        };

        self.keystore_repository
            .get_or_generate_key(realm.id)
            .await
            .map_err(|_| CoreError::RealmKeyNotFound)?;

        let client = match self
            .client_repository
            .get_by_client_id(config.default_client_id.clone(), realm.id)
            .await
        {
            Ok(client) => {
                tracing::info!(
                    "client {:} already exists",
                    config.default_client_id.clone()
                );

                client
            }
            Err(_) => {
                tracing::info!("createing client {:}", config.default_client_id.clone());
                let client = self
                    .client_repository
                    .create_client(CreateClientRequest {
                        realm_id: realm.id,
                        name: config.default_client_id.clone(),
                        client_id: config.default_client_id.clone(),
                        enabled: true,
                        protocol: "openid-connect".to_string(),
                        public_client: false,
                        service_account_enabled: false,
                        direct_access_grants_enabled: false,
                        client_type: "confidential".to_string(),
                        secret: Some(generate_random_string()),
                    })
                    .await
                    .map_err(|_| CoreError::CreateClientError)?;

                tracing::info!("client {:} created", config.default_client_id.clone());

                client
            }
        };

        let master_realm_client_id = format!("{}-realm", config.master_realm_name);

        let master_realm_client = match self
            .client_repository
            .get_by_client_id(master_realm_client_id.clone(), realm.id)
            .await
        {
            Ok(client) => {
                tracing::info!("client {:} created", master_realm_client_id.clone());
                client
            }
            Err(_) => {
                tracing::info!("creating client {:}", master_realm_client_id.clone());

                let client = self
                    .client_repository
                    .create_client(CreateClientRequest {
                        realm_id: realm.id,
                        name: master_realm_client_id.clone(),
                        client_id: master_realm_client_id.clone(),
                        enabled: true,
                        protocol: "openid-connect".to_string(),
                        public_client: false,
                        service_account_enabled: false,
                        direct_access_grants_enabled: true,
                        client_type: "confidential".to_string(),
                        secret: Some(generate_random_string()),
                    })
                    .await
                    .map_err(|_| CoreError::CreateClientError)?;

                tracing::info!("client {:} created", master_realm_client_id.clone());

                client
            }
        };

        let user = match self
            .user_repository
            .get_by_username(config.admin_username.clone(), realm.id)
            .await
        {
            Ok(user) => {
                let username = user.username.clone();
                tracing::info!("user {username:} already exists");
                user
            }
            Err(_) => {
                let client_id = config.default_client_id.clone();
                tracing::info!("Creating user for client {client_id:}");
                let user = self
                    .user_repository
                    .create_user(CreateUserRequest {
                        email: config.admin_email.clone(),
                        email_verified: true,
                        enabled: true,
                        firstname: config.admin_username.clone(),
                        lastname: config.admin_username.clone(),
                        realm_id: realm.id,
                        client_id: None,
                        username: config.admin_username.clone(),
                    })
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;

                tracing::info!("user {:} created", user.username);
                user
            }
        };

        let roles = self
            .role_repository
            .get_by_client_id(master_realm_client.id) // Updated to remove clone()
            .await
            .unwrap_or_default();
        let role = match roles
            .into_iter()
            .find(|r| r.name == master_realm_client_id.clone())
        {
            Some(role) => {
                tracing::info!("role {:} already exists", role.name);
                role
            }
            None => {
                let role = self
                    .role_repository
                    .create(CreateRoleRequest {
                        client_id: Some(master_realm_client.id),
                        name: master_realm_client_id.clone(),
                        permissions: Permissions::to_names(&[Permissions::ManageRealm]),
                        realm_id: realm.id,
                        description: None,
                    })
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;

                tracing::info!("role {:} created", master_realm_client_id.clone());
                role
            }
        };

        match self
            .user_role_repository
            .assign_role(user.id, role.id)
            .await
        {
            Ok(_) => {
                tracing::info!("role {:} assigned to user {:}", role.name, user.username);
            }
            Err(_) => {
                tracing::info!(
                    "role {:} already assigned to user {:}",
                    role.name,
                    user.username
                );
            }
        }

        let hash = self
            .hasher_repository
            .hash_password(&config.admin_password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        match self
            .credential_repository
            .create_credential(user.id, "password".to_string(), hash, "".into(), false)
            .await
        {
            Ok(_) => {
                tracing::info!("credential created for user {:}", user.username);
            }
            Err(_) => {
                tracing::info!("credential already exists for user {:}", user.username);
            }
        }

        let admin_redirect_patterns = vec![
            // Pattern regex pour accepter toutes les URLs sur localhost avec n'importe quel port
            "^http://localhost:[0-9]+/.*",
            "^/*",
            "http://localhost:3000/admin",
            "http://localhost:5173/admin",
        ];

        let existing_uris = self
            .redirect_uri_repository
            .get_by_client_id(client.id)
            .await
            .unwrap_or_default();

        for pattern in admin_redirect_patterns {
            let pattern_exists = existing_uris.iter().any(|uri| uri.value == pattern);

            if !pattern_exists {
                match self
                    .redirect_uri_repository
                    .create_redirect_uri(client.id, pattern.to_string(), true)
                    .await
                {
                    Ok(_) => {
                        tracing::info!("redirect uri created for client {:}", client.id);
                    }
                    Err(e) => {
                        tracing::error!(
                            "failed to create redirect uri for client {:}: {}",
                            client.id,
                            e
                        );
                    }
                }
            } else {
                tracing::info!("admin redirect URI already exists: {}", pattern);
            }
        }

        Ok(InitializationResult {
            master_realm_id: realm.id,
            admin_role_id: role.id,
            admin_user_id: user.id,
            default_client_id: client.id,
        })
    }
}
