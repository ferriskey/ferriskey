use std::sync::Arc;
use url::Url;

use ferriskey_security::jwt::ports::KeyStoreRepository;

use crate::domain::{
    client::{
        entities::ClientType,
        ports::{ClientRepository, RedirectUriRepository},
        value_objects::CreateClientRequest,
    },
    common::{
        entities::{InitializationResult, StartupConfig, app_errors::CoreError},
        generate_random_string,
        ports::CoreService,
    },
    credential::ports::CredentialRepository,
    crypto::HasherRepository,
    realm::ports::RealmRepository,
    role::{
        entities::permission::Permissions, ports::RoleRepository, value_objects::CreateRoleRequest,
    },
    user::{
        ports::{UserRepository, UserRoleRepository},
        value_objects::CreateUserRequest,
    },
};

#[derive(Clone, Debug)]
pub struct CoreServiceImpl<R, K, C, U, RO, UR, H, CR, RU>
where
    R: RealmRepository,
    K: KeyStoreRepository,
    C: ClientRepository,
    U: UserRepository,
    RO: RoleRepository,
    UR: UserRoleRepository,
    H: HasherRepository,
    CR: CredentialRepository,
    RU: RedirectUriRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) keystore_repository: Arc<K>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) redirect_uri_repository: Arc<RU>,
}

impl<R, K, C, U, RO, UR, H, CR, RU> CoreServiceImpl<R, K, C, U, RO, UR, H, CR, RU>
where
    R: RealmRepository,
    K: KeyStoreRepository,
    C: ClientRepository,
    U: UserRepository,
    RO: RoleRepository,
    UR: UserRoleRepository,
    H: HasherRepository,
    CR: CredentialRepository,
    RU: RedirectUriRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        keystore_repository: Arc<K>,
        client_repository: Arc<C>,
        user_repository: Arc<U>,
        role_repository: Arc<RO>,
        user_role_repository: Arc<UR>,
        hasher_repository: Arc<H>,
        credential_repository: Arc<CR>,
        redirect_uri_repository: Arc<RU>,
    ) -> Self {
        CoreServiceImpl {
            realm_repository,
            keystore_repository,
            client_repository,
            user_repository,
            role_repository,
            user_role_repository,
            hasher_repository,
            credential_repository,
            redirect_uri_repository,
        }
    }
}

impl<R, K, C, U, RO, UR, H, CR, RU> CoreService for CoreServiceImpl<R, K, C, U, RO, UR, H, CR, RU>
where
    R: RealmRepository,
    K: KeyStoreRepository,
    C: ClientRepository,
    U: UserRepository,
    RO: RoleRepository,
    UR: UserRoleRepository,
    H: HasherRepository,
    CR: CredentialRepository,
    RU: RedirectUriRepository,
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
                        client_type: ClientType::Confidential,
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
                        client_type: ClientType::Confidential,
                        secret: Some(generate_random_string()),
                    })
                    .await
                    .map_err(|_| CoreError::CreateClientError)?;

                tracing::info!("client {:} created", master_realm_client_id.clone());

                client
            }
        };

        match self
            .client_repository
            .get_by_client_id("admin-cli".to_string(), realm.id)
            .await
        {
            Ok(_) => {
                tracing::info!("client admin-cli already exists");
            }
            Err(_) => {
                tracing::info!("creating client admin-cli");
                self.client_repository
                    .create_client(CreateClientRequest {
                        realm_id: realm.id,
                        name: "admin-cli".to_string(),
                        client_id: "admin-cli".to_string(),
                        enabled: true,
                        protocol: "openid-connect".to_string(),
                        public_client: true,
                        service_account_enabled: false,
                        direct_access_grants_enabled: true,
                        client_type: ClientType::System,
                        secret: None,
                    })
                    .await
                    .map_err(|_| CoreError::CreateClientError)?;

                tracing::info!("client admin-cli created");
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

        let configured_base_url = config
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .and_then(|url_str| {
                match Url::parse(url_str) {
                    Ok(url) => {
                        // Validate that it has a scheme (http or https) and a host
                        if (url.scheme() == "http" || url.scheme() == "https")
                            && url.host().is_some()
                        {
                            Some(url)
                        } else {
                            tracing::warn!(
                                "Invalid base_url: must have http/https scheme and host - {}",
                                url_str
                            );
                            None
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse base_url '{}' as URL: {}", url_str, e);
                        None
                    }
                }
            });

        let admin_redirect_uris: Vec<String> = if let Some(base_url) = configured_base_url {
            let base_url_str = base_url.as_str().trim_end_matches('/').to_string();
            vec![
                format!(
                    "{}/realms/{}/authentication/callback",
                    base_url_str, config.master_realm_name
                ),
                format!(
                    "{}/realms/{}/authentication/login",
                    base_url_str, config.master_realm_name
                ),
            ]
        } else {
            // Fallback to localhost defaults for backward compatibility
            vec![
                format!(
                    "http://localhost:5555/realms/{}/authentication/callback",
                    config.master_realm_name
                ),
                format!(
                    "http://localhost:5555/realms/{}/authentication/login",
                    config.master_realm_name
                ),
                format!(
                    "http://localhost:3000/realms/{}/authentication/callback",
                    config.master_realm_name
                ),
                format!(
                    "http://localhost:3000/realms/{}/authentication/login",
                    config.master_realm_name
                ),
                format!(
                    "http://localhost:5173/realms/{}/authentication/callback",
                    config.master_realm_name
                ),
                format!(
                    "http://localhost:5173/realms/{}/authentication/login",
                    config.master_realm_name
                ),
            ]
        };

        let existing_uris = self
            .redirect_uri_repository
            .get_by_client_id(client.id)
            .await
            .unwrap_or_default();

        let callback_path = format!(
            "/realms/{}/authentication/callback",
            config.master_realm_name
        );
        let login_path = format!("/realms/{}/authentication/login", config.master_realm_name);

        for existing_uri in existing_uris.iter() {
            let matches_path = match Url::parse(&existing_uri.value) {
                Ok(url) => {
                    let path = url.path();
                    path == callback_path || path == login_path
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse existing URI '{}' for path matching: {}",
                        existing_uri.value,
                        e
                    );
                    false
                }
            };

            if admin_redirect_uris.contains(&existing_uri.value)
                && matches_path
                && !existing_uri.enabled
            {
                match self
                    .redirect_uri_repository
                    .update_enabled(existing_uri.id, true)
                    .await
                {
                    Ok(_) => {
                        tracing::info!("Re-enabled admin redirect URI: {}", existing_uri.value)
                    }
                    Err(e) => tracing::error!(
                        "Failed to re-enable redirect URI {}: {}",
                        existing_uri.value,
                        e
                    ),
                }
            } else if !admin_redirect_uris.contains(&existing_uri.value)
                && matches_path
                && existing_uri.enabled
            {
                match self
                    .redirect_uri_repository
                    .update_enabled(existing_uri.id, false)
                    .await
                {
                    Ok(_) => tracing::info!(
                        "Disabled obsolete admin redirect URI: {}",
                        existing_uri.value
                    ),
                    Err(e) => tracing::error!(
                        "Failed to disable obsolete redirect URI {}: {}",
                        existing_uri.value,
                        e
                    ),
                }
            }
        }

        for uri in admin_redirect_uris {
            let uri_exists = existing_uris
                .iter()
                .any(|existing_uri| existing_uri.value == uri);

            if !uri_exists {
                match self
                    .redirect_uri_repository
                    .create_redirect_uri(client.id, uri.to_string(), true)
                    .await
                {
                    Ok(_) => {
                        tracing::info!("Created admin redirect URI: {}", uri);
                    }
                    Err(e) => {
                        tracing::error!("Failed to create admin redirect URI {}: {}", uri, e);
                    }
                }
            } else {
                tracing::info!("admin redirect URI already exists: {}", uri);
            }
        }

        // Ensure security-admin-console exists in every realm, not just master.
        // This is idempotent: existing clients and URIs are skipped.
        // It also back-fills realms that were created before this seeding was added.
        let all_realms = self
            .realm_repository
            .fetch_realm()
            .await
            .unwrap_or_default();

        let console_redirect_patterns = [
            "^http://localhost:[0-9]+/.*",
            "^/*",
            "http://localhost:3000/admin",
            "http://localhost:5173/admin",
        ];

        for r in &all_realms {
            if r.id == realm.id {
                // master is already handled above
                continue;
            }

            let console_client = match self
                .client_repository
                .get_by_client_id(config.default_client_id.clone(), r.id)
                .await
            {
                Ok(c) => {
                    tracing::info!("security-admin-console already exists in realm {:}", r.name);
                    c
                }
                Err(_) => {
                    tracing::info!("creating security-admin-console for realm {:}", r.name);
                    match self
                        .client_repository
                        .create_client(CreateClientRequest {
                            realm_id: r.id,
                            name: config.default_client_id.clone(),
                            client_id: config.default_client_id.clone(),
                            enabled: true,
                            protocol: "openid-connect".to_string(),
                            public_client: false,
                            service_account_enabled: false,
                            direct_access_grants_enabled: false,
                            client_type: ClientType::Confidential,
                            secret: Some(generate_random_string()),
                        })
                        .await
                    {
                        Ok(c) => {
                            tracing::info!("security-admin-console created for realm {:}", r.name);
                            c
                        }
                        Err(e) => {
                            tracing::error!(
                                "failed to create security-admin-console for realm {:}: {:}",
                                r.name,
                                e
                            );
                            continue;
                        }
                    }
                }
            };

            let existing_uris = self
                .redirect_uri_repository
                .get_by_client_id(console_client.id)
                .await
                .unwrap_or_default();

            for pattern in &console_redirect_patterns {
                if !existing_uris.iter().any(|uri| &uri.value == pattern)
                    && let Err(e) = self
                        .redirect_uri_repository
                        .create_redirect_uri(console_client.id, pattern.to_string(), true)
                        .await
                {
                    tracing::error!(
                        "failed to create redirect URI '{}' for security-admin-console in realm '{}': {}",
                        pattern,
                        r.name,
                        e
                    );
                }
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
    use uuid::Uuid;

    use crate::domain::realm::entities::RealmId;
    use crate::domain::{
        authentication::value_objects::Identity,
        client::entities::{Client, ClientType, MaintenanceSessionStrategy},
        common::entities::app_errors::CoreError,
        realm::entities::Realm,
        role::entities::Role,
        user::entities::User,
    };

    pub fn create_test_user(realm_id: RealmId) -> User {
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
            roles: Some(Vec::new()),
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
            roles: Some(Vec::new()),
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
            roles: Some(Vec::new()),
        }
    }

    pub fn create_test_user_identity_with_realm(realm: &Realm) -> Identity {
        Identity::User(create_test_user_with_realm(realm))
    }

    pub fn create_test_user_with_params(
        realm_id: RealmId,
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
            roles: Some(Vec::new()),
        }
    }

    pub fn create_test_realm() -> Realm {
        Realm {
            id: RealmId::default(),
            name: "test-realm".to_string(),
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_test_realm_with_name(name: &str) -> Realm {
        Realm {
            id: RealmId::default(),
            name: name.to_string(),
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Crée un rôle de test
    pub fn create_test_role(realm_id: RealmId) -> Role {
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
        realm_id: RealmId,
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
    pub fn create_test_user_identity(realm_id: RealmId) -> Identity {
        Identity::User(create_test_user(realm_id))
    }

    pub fn create_test_client_identity(realm_id: RealmId) -> Identity {
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
            client_type: ClientType::Confidential,
            protocol: "openid-connect".to_string(),
            redirect_uris: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::default(),
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
}
