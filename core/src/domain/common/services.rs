use std::sync::Arc;

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
    webhook::ports::WebhookRepository,
};

#[derive(Clone)]
pub struct Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
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
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) auth_session_repository: Arc<AS>,
    pub(crate) redirect_uri_repository: Arc<RU>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) keystore_repository: Arc<KS>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) user_required_action_repository: Arc<URA>,
    pub(crate) health_check_repository: Arc<HC>,
    pub(crate) webhook_repository: Arc<W>,
    pub(crate) refresh_token_repository: Arc<RT>,
    pub(crate) recovery_code_repository: Arc<RC>,

    pub(crate) policy: FerriskeyPolicy<U, C, UR>,
}

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
    Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
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
        refresh_token_repository: RT,
        recovery_code_repository: RC,
    ) -> Self {
        let user_repo_arc = Arc::new(user_repository);
        let client_repo_arc = Arc::new(client_repository);
        let user_role_repo_arc = Arc::new(user_role_repository);

        let policy = FerriskeyPolicy::new(
            user_repo_arc.clone(),
            client_repo_arc.clone(),
            user_role_repo_arc.clone(),
        );

        Service {
            realm_repository: Arc::new(realm_repository),
            client_repository: client_repo_arc,
            user_repository: user_repo_arc,
            credential_repository: Arc::new(credential_repository),
            hasher_repository: Arc::new(hasher_repository),
            auth_session_repository: Arc::new(auth_session_repository),
            redirect_uri_repository: Arc::new(redirect_uri_repository),
            role_repository: Arc::new(role_repository),
            keystore_repository: Arc::new(keystore_repository),
            user_role_repository: user_role_repo_arc,
            user_required_action_repository: Arc::new(user_required_action_repository),
            health_check_repository: Arc::new(health_check_repository),
            webhook_repository: Arc::new(webhook_repository),
            refresh_token_repository: Arc::new(refresh_token_repository),
            recovery_code_repository: Arc::new(recovery_code_repository),

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

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC> CoreService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
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

        match self.realm_repository.get_realm_settings(realm.id).await? {
            None => {
                self.realm_repository
                    .create_realm_settings(realm.id, "RSA256".to_string())
                    .await?;
            }
            _ => {
                tracing::info!(
                    "realm settings already initialized for realm {:}",
                    realm.name
                );
            }
        };

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

#[cfg(test)]
pub mod tests {
    use std::panic;

    use chrono::Utc;
    use mockall::predicate::eq;
    use uuid::Uuid;

    use crate::domain::{
        authentication::{ports::MockAuthSessionRepository, value_objects::Identity},
        client::{
            entities::Client,
            ports::{MockClientRepository, MockRedirectUriRepository},
        },
        common::{entities::app_errors::CoreError, services::Service},
        credential::ports::MockCredentialRepository,
        crypto::ports::MockHasherRepository,
        health::ports::MockHealthCheckRepository,
        jwt::ports::{MockKeyStoreRepository, MockRefreshTokenRepository},
        realm::{entities::Realm, ports::MockRealmRepository},
        role::{
            entities::{Role, permission::Permissions},
            ports::MockRoleRepository,
        },
        trident::ports::MockRecoveryCodeRepository,
        user::{
            entities::User,
            ports::{MockUserRepository, MockUserRequiredActionRepository, MockUserRoleRepository},
        },
        webhook::ports::MockWebhookRepository,
    };

    pub type TestService = Service<
        MockRealmRepository,
        MockClientRepository,
        MockUserRepository,
        MockCredentialRepository,
        MockHasherRepository,
        MockAuthSessionRepository,
        MockRedirectUriRepository,
        MockRoleRepository,
        MockKeyStoreRepository,
        MockUserRoleRepository,
        MockUserRequiredActionRepository,
        MockHealthCheckRepository,
        MockWebhookRepository,
        MockRefreshTokenRepository,
        MockRecoveryCodeRepository,
    >;

    /// Macros pour créer des mocks async avec clonage automatique
    macro_rules! mock_async_with_clone_1_param {
        // Pattern pour 1 paramètre : retourne Ok(Some(value))
        ($value:expr) => {{
            let value_clone = $value.clone();
            move |_| {
                let value = value_clone.clone();
                Box::pin(async move { Ok(Some(value)) })
            }
        }};

        // Pattern pour 1 paramètre : retourne Ok(value) directement
        ($value:expr, direct) => {{
            let value_clone = $value.clone();
            move |_| {
                let value = value_clone.clone();
                Box::pin(async move { Ok(value) })
            }
        }};
    }

    macro_rules! mock_async_with_clone_2_params {
        // Pattern pour 2 paramètres : retourne Ok(value) directement
        ($value:expr, direct) => {{
            let value_clone = $value.clone();
            move |_, _| {
                let value = value_clone.clone();
                Box::pin(async move { Ok(value) })
            }
        }};
    }

    /// Macro pour créer des mocks async simples sans clonage
    macro_rules! mock_async {
        // Pattern de base : |_|
        ($expr:expr) => {
            move |_| Box::pin(async move { $expr })
        };

        // Pattern avec 2 paramètres : |param1, param2|
        ($expr:expr, two_params) => {
            move |_, _| Box::pin(async move { $expr })
        };
    }

    /// Builder pattern pour créer facilement des services de test avec des mocks configurés
    pub struct ServiceTestBuilder {
        realm_repo: MockRealmRepository,
        client_repo: MockClientRepository,
        user_repo: MockUserRepository,
        credential_repo: MockCredentialRepository,
        hasher_repo: MockHasherRepository,
        auth_session_repo: MockAuthSessionRepository,
        redirect_uri_repo: MockRedirectUriRepository,
        role_repo: MockRoleRepository,
        keystore_repo: MockKeyStoreRepository,
        user_role_repo: MockUserRoleRepository,
        user_required_action_repo: MockUserRequiredActionRepository,
        health_check_repo: MockHealthCheckRepository,
        webhook_repo: MockWebhookRepository,
        refresh_token_repo: MockRefreshTokenRepository,
        recovery_code_repo: MockRecoveryCodeRepository,
    }

    impl Default for ServiceTestBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ServiceTestBuilder {
        pub fn new() -> Self {
            Self {
                realm_repo: MockRealmRepository::new(),
                client_repo: MockClientRepository::new(),
                user_repo: MockUserRepository::new(),
                credential_repo: MockCredentialRepository::new(),
                hasher_repo: MockHasherRepository::new(),
                auth_session_repo: MockAuthSessionRepository::new(),
                redirect_uri_repo: MockRedirectUriRepository::new(),
                role_repo: MockRoleRepository::new(),
                keystore_repo: MockKeyStoreRepository::new(),
                user_role_repo: MockUserRoleRepository::new(),
                user_required_action_repo: MockUserRequiredActionRepository::new(),
                health_check_repo: MockHealthCheckRepository::new(),
                webhook_repo: MockWebhookRepository::new(),
                refresh_token_repo: MockRefreshTokenRepository::new(),
                recovery_code_repo: MockRecoveryCodeRepository::new(),
            }
        }

        pub fn with_realm_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockRealmRepository),
        {
            setup(&mut self.realm_repo);
            self
        }

        /// Configure le mock ClientRepository
        pub fn with_client_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockClientRepository),
        {
            setup(&mut self.client_repo);
            self
        }

        /// Configure le mock UserRepository
        pub fn with_user_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockUserRepository),
        {
            setup(&mut self.user_repo);
            self
        }

        /// Configure le mock RoleRepository
        pub fn with_role_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockRoleRepository),
        {
            setup(&mut self.role_repo);
            self
        }

        /// Configure le mock UserRoleRepository
        pub fn with_user_role_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockUserRoleRepository),
        {
            setup(&mut self.user_role_repo);
            self
        }

        /// Configure le mock CredentialRepository
        pub fn with_credential_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockCredentialRepository),
        {
            setup(&mut self.credential_repo);
            self
        }

        /// Configure le mock HasherRepository
        pub fn with_hasher_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockHasherRepository),
        {
            setup(&mut self.hasher_repo);
            self
        }

        /// Configure le mock AuthSessionRepository
        pub fn with_auth_session_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockAuthSessionRepository),
        {
            setup(&mut self.auth_session_repo);
            self
        }

        /// Configure le mock KeyStoreRepository
        pub fn with_keystore_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockKeyStoreRepository),
        {
            setup(&mut self.keystore_repo);
            self
        }

        /// Configure le mock WebhookRepository
        pub fn with_webhook_repo<F>(mut self, setup: F) -> Self
        where
            F: FnOnce(&mut MockWebhookRepository),
        {
            setup(&mut self.webhook_repo);
            self
        }

        // === HELPERS POUR CAS DE TEST COURANTS ===

        /// Configure un realm lookup réussi
        pub fn with_successful_realm_lookup(self, realm_name: &str, realm: Realm) -> Self {
            self.with_realm_repo(|repo| {
                repo.expect_get_by_name()
                    .with(eq(realm_name.to_string()))
                    .once()
                    .returning(mock_async_with_clone_1_param!(realm));
            })
        }

        /// Configure un realm non trouvé
        pub fn with_realm_not_found(self, realm_name: &str) -> Self {
            self.with_realm_repo(|repo| {
                repo.expect_get_by_name()
                    .with(eq(realm_name.to_string()))
                    .once()
                    .returning(mock_async!(Ok(None)));
            })
        }

        /// Configure une erreur lors de la recherche de realm
        pub fn with_realm_lookup_error(self, realm_name: &str) -> Self {
            self.with_realm_repo(|repo| {
                repo.expect_get_by_name()
                    .with(eq(realm_name.to_string()))
                    .once()
                    .returning(mock_async!(Err(
                        crate::domain::common::entities::app_errors::CoreError::InternalServerError
                    )));
            })
        }

        /// Configure un role lookup réussi
        pub fn with_successful_role_lookup(self, role_id: Uuid, role: Role) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_get_by_id()
                    .with(eq(role_id))
                    .once()
                    .returning(mock_async_with_clone_1_param!(role));
            })
        }

        /// Configure un role non trouvé
        pub fn with_role_not_found(self, role_id: Uuid) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_get_by_id()
                    .with(eq(role_id))
                    .once()
                    .returning(mock_async!(Ok(None)));
            })
        }

        /// Configure une erreur lors de la recherche de role
        pub fn with_role_lookup_error(self, role_id: Uuid) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_get_by_id()
                    .with(eq(role_id))
                    .once()
                    .returning(mock_async!(Err(
                        crate::domain::common::entities::app_errors::CoreError::InternalServerError
                    )));
            })
        }

        /// Configure les rôles d'un utilisateur
        pub fn with_user_roles(self, user_id: Uuid, roles: Vec<Role>) -> Self {
            self.with_user_role_repo(|repo| {
                repo.expect_get_user_roles()
                    .with(eq(user_id))
                    .once()
                    .returning(mock_async_with_clone_1_param!(roles, direct));
            })
        }

        /// Configure un utilisateur sans rôles
        pub fn with_no_user_roles(self, user_id: Uuid) -> Self {
            self.with_user_roles(user_id, vec![])
        }

        /// Configure une erreur lors de la récupération des rôles utilisateur
        pub fn with_user_roles_error(self, user_id: Uuid) -> Self {
            self.with_user_role_repo(|repo| {
                repo.expect_get_user_roles()
                    .with(eq(user_id))
                    .once()
                    .returning(mock_async!(Err(
                        crate::domain::common::entities::app_errors::CoreError::Forbidden(
                            "Access denied".to_string()
                        )
                    )));
            })
        }

        /// Configure un user lookup réussi par client_id (pour service accounts)
        pub fn with_successful_user_by_client_lookup(self, client_id: Uuid, user: User) -> Self {
            self.with_user_repo(|repo| {
                repo.expect_get_by_client_id()
                    .with(eq(client_id))
                    .once()
                    .returning(mock_async_with_clone_1_param!(user, direct));
            })
        }

        /// Configure un user lookup réussi par ID
        pub fn with_successful_user_lookup(self, user_id: Uuid, user: User) -> Self {
            self.with_user_repo(|repo| {
                repo.expect_get_by_id()
                    .with(eq(user_id))
                    .once()
                    .returning(mock_async_with_clone_1_param!(user, direct));
            })
        }

        /// Configure un user non trouvé
        pub fn with_user_not_found(self, user_id: Uuid) -> Self {
            self.with_user_repo(|repo| {
                repo.expect_get_by_id()
                    .with(eq(user_id))
                    .once()
                    .returning(mock_async!(Err(
                        crate::domain::common::entities::app_errors::CoreError::NotFound
                    )));
            })
        }

        /// Configure un client lookup réussi (2 paramètres : client_id, realm_id)
        pub fn with_successful_client_lookup(
            self,
            client_id: &str,
            realm_id: Uuid,
            client: Client,
        ) -> Self {
            self.with_client_repo(|repo| {
                repo.expect_get_by_client_id()
                    .with(eq(client_id.to_string()), eq(realm_id))
                    .once()
                    .returning(mock_async_with_clone_2_params!(client, direct));
            })
        }

        /// Configure une liste de roles par realm_id
        pub fn with_roles_by_realm(self, realm_id: Uuid, roles: Vec<Role>) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_find_by_realm_id()
                    .with(eq(realm_id))
                    .once()
                    .returning(mock_async_with_clone_1_param!(roles, direct));
            })
        }

        /// Configure une création de role réussie
        pub fn with_successful_role_creation(self, role: Role) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_create()
                    .once()
                    .returning(mock_async_with_clone_1_param!(role, direct));
            })
        }

        /// Configure une mise à jour de role réussie (2 paramètres : id, payload)
        pub fn with_successful_role_update(self, role_id: Uuid, updated_role: Role) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_update_by_id()
                    .with(eq(role_id), mockall::predicate::always())
                    .once()
                    .returning(mock_async_with_clone_2_params!(updated_role, direct));
            })
        }

        /// Configure une suppression de role réussie
        pub fn with_successful_role_deletion(self, role_id: Uuid) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_delete_by_id()
                    .with(eq(role_id))
                    .once()
                    .returning(mock_async!(Ok(())));
            })
        }

        /// Configure une mise à jour des permissions de role réussie (2 paramètres : id, payload)
        pub fn with_successful_role_permissions_update(
            self,
            role_id: Uuid,
            updated_role: Role,
        ) -> Self {
            self.with_role_repo(|repo| {
                repo.expect_update_permissions_by_id()
                    .with(eq(role_id), mockall::predicate::always())
                    .once()
                    .returning(mock_async_with_clone_2_params!(updated_role, direct));
            })
        }

        /// Configure un user lookup par username réussi (2 paramètres : username, realm_id)
        pub fn with_successful_user_lookup_by_username(
            self,
            username: &str,
            realm_id: Uuid,
            user: User,
        ) -> Self {
            self.with_user_repo(|repo| {
                repo.expect_get_by_username()
                    .with(eq(username.to_string()), eq(realm_id))
                    .once()
                    .returning(mock_async_with_clone_2_params!(user, direct));
            })
        }

        /// Configure une création d'utilisateur réussie
        pub fn with_successful_user_creation(self, user: User) -> Self {
            self.with_user_repo(|repo| {
                repo.expect_create_user()
                    .once()
                    .returning(mock_async_with_clone_1_param!(user, direct));
            })
        }

        /// Configure une mise à jour d'utilisateur réussie (2 paramètres : user_id, payload)
        pub fn with_successful_user_update(self, user_id: Uuid, updated_user: User) -> Self {
            self.with_user_repo(|repo| {
                repo.expect_update_user()
                    .with(eq(user_id), mockall::predicate::always())
                    .once()
                    .returning(mock_async_with_clone_2_params!(updated_user, direct));
            })
        }

        /// Configure une suppression d'utilisateur réussie
        pub fn with_successful_user_deletion(self, user_id: Uuid, deleted_count: u64) -> Self {
            self.with_user_repo(|repo| {
                repo.expect_delete_user()
                    .with(eq(user_id))
                    .once()
                    .returning(move |_| Box::pin(async move { Ok(deleted_count) }));
            })
        }

        /// Configure un client non trouvé
        pub fn with_client_not_found(self, client_id: &str, realm_id: Uuid) -> Self {
            self.with_client_repo(|repo| {
                repo.expect_get_by_client_id()
                    .with(eq(client_id.to_string()), eq(realm_id))
                    .once()
                    .returning(mock_async!(
                        Err(crate::domain::common::entities::app_errors::CoreError::NotFound),
                        two_params
                    ));
            })
        }

        /// Configure une création de realm réussie
        pub fn with_successful_realm_creation(
            self,
            realm_name: &str,
            created_realm: Realm,
        ) -> Self {
            self.with_realm_repo(|repo| {
                repo.expect_create_realm()
                    .with(eq(realm_name.to_string()))
                    .once()
                    .returning(mock_async_with_clone_1_param!(created_realm, direct));
            })
        }

        /// Configure une mise à jour de realm réussie (2 paramètres : old_name, new_name)
        pub fn with_successful_realm_update(
            self,
            old_name: &str,
            new_name: &str,
            updated_realm: Realm,
        ) -> Self {
            self.with_realm_repo(|repo| {
                repo.expect_update_realm()
                    .with(eq(old_name.to_string()), eq(new_name.to_string()))
                    .once()
                    .returning(mock_async_with_clone_2_params!(updated_realm, direct));
            })
        }

        /// Configure une suppression de realm réussie
        pub fn with_successful_realm_deletion(self, realm_name: &str) -> Self {
            self.with_realm_repo(|repo| {
                repo.expect_delete_by_name()
                    .with(eq(realm_name.to_string()))
                    .once()
                    .returning(mock_async!(Ok(())));
            })
        }

        pub fn with_user_view_roles_permissions(self, user_id: Uuid, realm_id: Uuid) -> Self {
            let role_with_view_permission = create_test_role_with_params(
                realm_id,
                "viewer-role",
                vec![Permissions::ViewRoles.name()],
                None,
            );

            self.with_user_roles(user_id, vec![role_with_view_permission])
        }

        /// Helper pour créer un rôle avec les permissions de manage users
        pub fn with_user_manage_users_permissions(self, user_id: Uuid, realm_id: Uuid) -> Self {
            let role_with_manage_permission = create_test_role_with_params(
                realm_id,
                "admin-role",
                vec!["manage_users".to_string()],
                None,
            );
            self.with_user_roles(user_id, vec![role_with_manage_permission])
        }

        /// Helper pour créer un rôle avec les permissions de manage realm
        pub fn with_user_manage_realm_permissions(self, user_id: Uuid, realm_id: Uuid) -> Self {
            let role_with_manage_realm_permission = create_test_role_with_params(
                realm_id,
                "realm-admin-role",
                vec!["manage_realm".to_string()],
                None,
            );
            self.with_user_roles(user_id, vec![role_with_manage_realm_permission])
        }

        /// Helper pour créer un utilisateur avec toutes les permissions
        pub fn with_user_all_permissions(self, user_id: Uuid, realm_id: Uuid) -> Self {
            let super_admin_role = create_test_role_with_params(
                realm_id,
                "super-admin",
                vec![
                    "manage_realm".to_string(),
                    "manage_users".to_string(),
                    "view_roles".to_string(),
                    "manage_roles".to_string(),
                ],
                None,
            );
            self.with_user_roles(user_id, vec![super_admin_role])
        }

        pub fn build(self) -> TestService {
            Service::new(
                self.realm_repo,
                self.client_repo,
                self.user_repo,
                self.credential_repo,
                self.hasher_repo,
                self.auth_session_repo,
                self.redirect_uri_repo,
                self.role_repo,
                self.keystore_repo,
                self.user_role_repo,
                self.user_required_action_repo,
                self.health_check_repo,
                self.webhook_repo,
                self.refresh_token_repo,
                self.recovery_code_repo,
            )
        }
    }

    pub fn create_test_user(realm_id: Uuid) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id,
            username: "test_user".to_string(),
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            realm: None,
            client_id: None,
            enabled: true,
            required_actions: Vec::new(),
            roles: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_test_user_with_realm(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            realm_id: realm.id,
            realm: Some(realm.clone()),
            client_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled: true,
            email_verified: true,
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            required_actions: Vec::new(),
            roles: Vec::new(),
        }
    }

    pub fn create_test_user_with_params_and_realm(
        realm: &Realm,
        username: &str,
        email: String,
        enabled: bool,
    ) -> User {
        User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email,
            realm_id: realm.id,
            realm: Some(realm.clone()),
            client_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled,
            email_verified: true,
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            required_actions: Vec::new(),
            roles: Vec::new(),
        }
    }

    pub fn create_test_user_identity_with_realm(realm: &Realm) -> Identity {
        Identity::User(create_test_user_with_realm(realm))
    }

    pub fn create_test_user_with_params(
        realm_id: Uuid,
        firstname: &str,
        lastname: &str,
        username: &str,
        email: &str,
        enabled: bool,
    ) -> User {
        User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.to_string(),
            firstname: firstname.to_string(),
            lastname: lastname.to_string(),
            realm_id,
            realm: None,
            client_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled,
            email_verified: true,
            required_actions: Vec::new(),
            roles: Vec::new(),
        }
    }

    pub fn create_test_realm() -> Realm {
        Realm {
            id: Uuid::new_v4(),
            name: "test-realm".to_string(),
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_test_realm_with_name(name: &str) -> Realm {
        Realm {
            id: Uuid::new_v4(),
            name: name.to_string(),
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Crée un rôle de test
    pub fn create_test_role(realm_id: Uuid) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "test-role".to_string(),
            description: Some("Test role description".to_string()),
            permissions: vec!["read:users".to_string(), "write:users".to_string()],
            realm_id,
            client_id: None,
            client: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Crée un rôle de test avec des paramètres personnalisés
    pub fn create_test_role_with_params(
        realm_id: Uuid,
        name: &str,
        permissions: Vec<String>,
        client_id: Option<Uuid>,
    ) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: Some(format!("{} description", name)),
            permissions,
            realm_id,
            client_id,
            client: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Crée une identité utilisateur de test
    pub fn create_test_user_identity(realm_id: Uuid) -> Identity {
        Identity::User(create_test_user(realm_id))
    }

    pub fn create_test_client_identity(realm_id: Uuid) -> Identity {
        let client = Client {
            id: Uuid::new_v4(),
            client_id: "test-client".to_string(),
            secret: Some("secret".to_string()),
            name: "Test Client".to_string(),
            realm_id,
            enabled: true,
            public_client: false,
            direct_access_grants_enabled: true,
            service_account_enabled: true,
            client_type: "confidential".to_string(),
            protocol: "openid-connect".to_string(),
            redirect_uris: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Identity::Client(client)
    }

    pub fn assert_core_erro(
        result: Result<impl std::fmt::Debug, CoreError>,
        expected_error: CoreError,
    ) {
        match result {
            Err(actual_error) => {
                assert!(
                    std::mem::discriminant(&actual_error)
                        == std::mem::discriminant(&expected_error),
                    "Expected error {:?}, but got {:?}",
                    expected_error,
                    actual_error
                );
            }
            Ok(value) => panic!(
                "Expected error {:?}, but got Ok({:?})",
                expected_error, value
            ),
        }
    }

    pub fn assert_success<T: std::fmt::Debug, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("Expected success, but got error: {:?}", error),
        }
    }

    #[cfg(test)]
    mod builder_tests {
        use crate::domain::common::services::tests::{
            ServiceTestBuilder, create_test_realm, create_test_role, create_test_user_with_realm,
        };

        #[test]
        fn test_service_builder_create_service() {
            let service = ServiceTestBuilder::new().build();

            assert!(std::ptr::addr_of!(service).is_aligned());
        }

        #[test]
        fn test_create_test_entities() {
            let realm = create_test_realm();
            let user = create_test_user_with_realm(&realm);
            let role = create_test_role(realm.id);

            assert_eq!(user.realm_id, realm.id);
            assert_eq!(role.realm_id, realm.id);
            assert_eq!(realm.name, "test-realm");
        }
    }
}
