use std::sync::Arc;

use tracing::info;

use crate::{
    application::http::client::validators::{CreateClientValidator, CreateRedirectUriValidator},
    domain::{
        client::{
            ports::{client_service::ClientService, redirect_uri_service::RedirectUriService},
            services::{
                client_service::DefaultClientService,
                redirect_uri_service::DefaultRedirectUriService,
            },
        },
        credential::{
            ports::credential_service::CredentialService,
            services::credential_service::DefaultCredentialService,
        },
        jwt::{ports::jwt_service::JwtService, services::jwt_service::DefaultJwtService},
        realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
        role::{
            entities::{CreateRoleDto, permission::Permissions},
            ports::RoleService,
            services::DefaultRoleService,
        },
        user::{
            dtos::user_dto::CreateUserDto,
            ports::{user_role_service::UserRoleService, user_service::UserService},
            services::{
                user_role_service::DefaultUserRoleService, user_service::DefaultUserService,
            },
        },
    },
    env::Env,
};

use crate::domain::mediator::ports::mediator_service::MediatorService;

pub type DefaultMediatorService = MediatorServiceImpl;

#[derive(Clone)]
pub struct MediatorServiceImpl {
    pub env: Arc<Env>,
    pub client_service: Arc<DefaultClientService>,
    pub realm_service: Arc<DefaultRealmService>,
    pub user_service: Arc<DefaultUserService>,
    pub credential_service: Arc<DefaultCredentialService>,
    pub redirect_uri_service: DefaultRedirectUriService,
    pub role_service: DefaultRoleService,
    pub user_role_service: DefaultUserRoleService,
    pub jwt_service: Arc<DefaultJwtService>,
}

impl MediatorServiceImpl {
    pub fn new(
        env: Arc<Env>,
        client_service: Arc<DefaultClientService>,
        realm_service: Arc<DefaultRealmService>,
        user_service: Arc<DefaultUserService>,
        credential_service: Arc<DefaultCredentialService>,
        redirect_uri_service: DefaultRedirectUriService,
        role_service: DefaultRoleService,
        user_role_service: DefaultUserRoleService,
        jwt_service: Arc<DefaultJwtService>,
    ) -> Self {
        Self {
            env,
            client_service,
            realm_service,
            user_service,
            credential_service,
            redirect_uri_service,
            role_service,
            user_role_service,
            jwt_service,
        }
    }
}

impl MediatorService for MediatorServiceImpl {
    async fn initialize_master_realm(&self) -> Result<(), anyhow::Error> {
        info!("Introspecting master realm");

        let realm = match self.realm_service.get_by_name("master".to_string()).await {
            Ok(realm) => {
                info!("Master realm already exists");
                realm
            }
            Err(_) => {
                info!("Creating master realm");
                self.realm_service
                    .create_realm("master".to_string())
                    .await?
            }
        };

        let _ = self.jwt_service.retrieve_realm_rsa_keys(&realm).await?;

        let client_id = "security-admin-console".to_string();

        let schema = CreateClientValidator {
            client_id: client_id.clone(),
            enabled: true,
            name: "security-admin-console".to_string(),
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: false,
            client_type: "confidential".to_string(),
        };

        let _client = match self
            .client_service
            .create_client(schema, realm.name.clone())
            .await
        {
            Ok(client) => {
                info!("client {:} created", client_id.clone());
                client
            }
            Err(_) => {
                info!("client {:} already exists", client_id.clone());
                self.client_service
                    .get_by_client_id(client_id.clone(), realm.id)
                    .await?
            }
        };

        let master_realm_client = match self
            .client_service
            .create_client(
                CreateClientValidator {
                    client_id: "master-realm".to_string(),
                    enabled: true,
                    name: "master-realm".to_string(),
                    protocol: "openid-connect".to_string(),
                    public_client: false,
                    service_account_enabled: false,
                    client_type: "confidential".to_string(),
                },
                realm.name.clone(),
            )
            .await
        {
            Ok(client) => {
                info!("client {:} created", client_id.clone());
                client
            }
            Err(_) => {
                info!("client {:} already exists", client_id.clone());
                self.client_service
                    .get_by_client_id("master-realm".to_string(), realm.id)
                    .await?
            }
        };

        // Initialize redirect URIs for the admin client
        self.initialize_admin_redirect_uris().await?;

        let user = match self
            .user_service
            .create_user(CreateUserDto {
                email: self.env.admin_email.clone(),
                email_verified: true,
                enabled: true,
                firstname: self.env.admin_username.clone(),
                lastname: self.env.admin_username.clone(),
                realm_id: realm.id,
                client_id: None,
                username: self.env.admin_username.clone(),
            })
            .await
        {
            Ok(user) => {
                info!("user {:} created", user.username);
                user
            }
            Err(_) => {
                let user = self
                    .user_service
                    .get_by_username(self.env.admin_username.clone(), realm.id)
                    .await?;
                info!("user {:} already exists", self.env.admin_username.clone());
                user
            }
        };

        let role = match self
            .role_service
            .create(CreateRoleDto {
                client_id: Some(master_realm_client.id),
                name: "master-realm".to_string(),
                permissions: Permissions::to_names(&[Permissions::ManageRealm]),
                realm_id: realm.id,
                description: None,
            })
            .await
        {
            Ok(role) => role,
            Err(_) => {
                info!("role {:} already exists", "master-realm");
                self.role_service
                    .find_by_name("master-realm".to_string(), realm.id)
                    .await?
            }
        };

        self.user_role_service
            .assign_role("master".to_string(), user.id, role.id)
            .await?;

        let _ = match self
            .credential_service
            .create_password_credential(user.id, self.env.admin_password.clone(), "".to_string())
            .await
        {
            Ok(credential) => {
                info!("credential {:} created", credential.id);
                credential
            }
            Err(_) => {
                info!(
                    "credential {:} already exists",
                    self.env.admin_username.clone()
                );
                return Ok(());
            }
        };

        Ok(())
    }

    async fn initialize_admin_redirect_uris(&self) -> Result<(), anyhow::Error> {
        info!("Initializing admin redirect URIs");

        // Récupération du realm master
        let realm = self.realm_service.get_by_name("master".to_string()).await?;

        // Récupération du client admin
        let client_id = "security-admin-console".to_string();
        let client = self
            .client_service
            .get_by_client_id(client_id.clone(), realm.id)
            .await?;

        // Configuration des patterns de redirection pour l'admin console
        let admin_redirect_patterns = vec![
            // Pattern regex pour accepter toutes les URLs sur localhost avec n'importe quel port
            "^http://localhost:[0-9]+/.*",
            "^/*",
            "http://localhost:3000/admin",
            "http://localhost:5173/admin",
        ];

        // Vérification des URIs existantes
        let existing_uris = match self.redirect_uri_service.get_by_client_id(client.id).await {
            Ok(uris) => uris,
            Err(_) => Vec::new(),
        };

        for pattern in admin_redirect_patterns {
            let pattern_exists = existing_uris.iter().any(|uri| uri.value == pattern);

            if !pattern_exists {
                info!("Adding admin redirect URI pattern: {}", pattern);

                let redirect_schema = CreateRedirectUriValidator {
                    value: pattern.to_string(),
                    enabled: true,
                };

                match self
                    .redirect_uri_service
                    .add_redirect_uri(redirect_schema, realm.name.clone(), client.id)
                    .await
                {
                    Ok(_) => info!("Successfully added admin redirect URI: {}", pattern),
                    Err(e) => info!("Failed to add admin redirect URI {}: {}", pattern, e),
                }
            } else {
                info!("Admin redirect URI already exists: {}", pattern);
            }
        }

        Ok(())
    }
}
