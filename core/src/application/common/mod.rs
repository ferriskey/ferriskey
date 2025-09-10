use crate::{
    application::{
        authentication::services::AuthenticateFactory, common::permissions::FerriskeyPolicy,
    },
    domain::{
        authentication::services::grant_type_service::GrantTypeStrategies,
        client::{
            ports::{ClientRepository, RedirectUriRepository},
            value_objects::CreateClientRequest,
        },
        common::{
            AppConfig, FerriskeyConfig,
            entities::{InitializationResult, StartupConfig, app_errors::CoreError},
            generate_random_string,
            ports::CoreService,
        },
        credential::ports::CredentialRepository,
        crypto::ports::HasherRepository,
        jwt::{ports::KeyStoreRepository, services::JwtServiceImpl},
        realm::ports::RealmRepository,
        role::{
            entities::permission::Permissions, ports::RoleRepository,
            value_objects::CreateRoleRequest,
        },
        user::{
            ports::{UserRepository, UserRoleRepository},
            value_objects::CreateUserRequest,
        },
    },
    infrastructure::{
        auth_session::AuthSessionRepoAny,
        client::repositories::{ClientRepoAny, RedirectUriRepoAny},
        credential::CredentialRepoAny,
        hasher::HasherRepoAny,
        health::HealthCheckRepoAny,
        jwt::KeyStoreRepoAny,
        realm::repositories::RealmRepoAny,
        refresh_token::RefreshTokenRepoAny,
        repositories::build_repos_from_env,
        role::repositories::RoleRepoAny,
        user::{
            UserRepoAny,
            repositories::{
                user_required_action_repository::UserRequiredActionRepoAny,
                user_role_repository::UserRoleRepoAny,
            },
        },
        webhook::repositories::{
            webhook_notifier_repository::WebhookNotifierRepoAny, webhook_repository::WebhookRepoAny,
        },
    },
};

pub mod permissions;
pub mod policies;

pub type DefaultJwtService = JwtServiceImpl<RefreshTokenRepoAny, KeyStoreRepoAny, RealmRepoAny>;

#[derive(Clone)]
pub struct FerriskeyService {
    //pub(crate) config: FerriskeyConfig,
    pub(crate) realm_repository: RealmRepoAny,
    pub(crate) client_repository: ClientRepoAny,
    pub(crate) user_repository: UserRepoAny,
    pub(crate) credential_repository: CredentialRepoAny,
    pub(crate) hasher_repository: HasherRepoAny,
    pub(crate) auth_session_repository: AuthSessionRepoAny,
    pub(crate) redirect_uri_repository: RedirectUriRepoAny,
    pub(crate) role_repository: RoleRepoAny,
    pub(crate) keystore_repository: KeyStoreRepoAny,
    pub(crate) user_role_repository: UserRoleRepoAny,
    pub(crate) user_required_action_repository: UserRequiredActionRepoAny,
    pub(crate) health_check_repository: HealthCheckRepoAny,
    pub(crate) webhook_repository: WebhookRepoAny,
    pub(crate) policy: FerriskeyPolicy,
    pub(crate) webhook_notifier_repository: WebhookNotifierRepoAny,
    pub(crate) grant_type_strategies: GrantTypeStrategies,
    pub(crate) authenticate_factory: AuthenticateFactory,
}

impl FerriskeyService {
    pub async fn new(config: FerriskeyConfig) -> Result<Self, anyhow::Error> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.database.username,
            config.database.password,
            config.database.host,
            config.database.port,
            config.database.name
        );
        let repos = build_repos_from_env(AppConfig { database_url }).await?;

        let policy = FerriskeyPolicy::new(
            repos.user_repository.clone(),
            repos.client_repository.clone(),
            repos.user_role_repository.clone(),
        );

        let grant_type_strategies = GrantTypeStrategies::new(
            repos.credential_repository.clone(),
            repos.hasher_repository.clone(),
            repos.auth_session_repository.clone(),
            repos.user_repository.clone(),
            repos.keystore_repository.clone(),
            repos.refresh_token_repository.clone(),
            repos.client_repository.clone(),
        );

        let jwt_service = DefaultJwtService::new(
            repos.refresh_token_repository.clone(),
            repos.keystore_repository.clone(),
            repos.realm_repository.clone(),
        );

        let authenticate_factory = AuthenticateFactory::new(
            repos.auth_session_repository.clone(),
            repos.user_repository.clone(),
            repos.realm_repository.clone(),
            repos.client_repository.clone(),
            repos.credential_repository.clone(),
            repos.hasher_repository.clone(),
            jwt_service,
        );

        Ok(FerriskeyService {
            realm_repository: repos.realm_repository,
            client_repository: repos.client_repository,
            user_repository: repos.user_repository,
            credential_repository: repos.credential_repository,
            hasher_repository: repos.hasher_repository,
            auth_session_repository: repos.auth_session_repository,
            redirect_uri_repository: repos.redirect_uri_repository,
            role_repository: repos.role_repository,
            keystore_repository: repos.keystore_repository,
            user_role_repository: repos.user_role_repository,
            user_required_action_repository: repos.user_required_action_repository,
            health_check_repository: repos.health_check_repository,
            webhook_repository: repos.webhook_repository,
            webhook_notifier_repository: repos.webhook_notifier_repository,

            policy,
            grant_type_strategies,
            authenticate_factory,
        })
    }
}

impl CoreService for FerriskeyService {
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
                    .await
                    .map_err(|_| CoreError::InvalidRealm)?;

                tracing::info!("{} realm created", config.master_realm_name);
                realm
            }
            Err(_) => {
                tracing::info!("creating master realm");
                let realm = self
                    .realm_repository
                    .create_realm(config.master_realm_name.clone())
                    .await
                    .map_err(|_| CoreError::InvalidRealm)?;

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
