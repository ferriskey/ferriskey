use std::vec;

use crate::application::common::services::{
    DefaultClientService, DefaultCredentialService, DefaultJwtService, DefaultRealmService,
    DefaultRedirectUriService, DefaultRoleService, DefaultUserRoleService, DefaultUserService,
};
use crate::domain::{
    client::{
        ports::{ClientService, RedirectUriService},
        value_objects::{CreateClientRequest, CreateRedirectUriRequest},
    },
    common::generate_random_string,
    credential::ports::CredentialService,
    jwt::ports::JwtService,
    realm::ports::RealmService,
    role::{
        entities::permission::Permissions, ports::RoleService, value_objects::CreateRoleRequest,
    },
    user::{
        ports::{UserRoleService, UserService},
        value_objects::CreateUserRequest,
    },
};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StartupConfig {
    pub master_realm_name: String,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password: String,
    pub default_client_id: String,
}

#[derive(Debug, Clone)]
pub struct InitializationResult {
    pub master_realm_id: Uuid,
    pub admin_user_id: Uuid,
    pub admin_role_id: Uuid,
    pub default_client_id: Uuid,
}

pub struct StartupOrchestrator {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    role_service: DefaultRoleService,
    jwt_service: DefaultJwtService,
    user_role_service: DefaultUserRoleService,
    credential_service: DefaultCredentialService,
    redirect_uri_service: DefaultRedirectUriService,
}

pub struct StartupOrchestratorBuilder {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub role_service: DefaultRoleService,
    pub jwt_service: DefaultJwtService,
    pub user_role_service: DefaultUserRoleService,
    pub credential_service: DefaultCredentialService,
    pub redirect_uri_service: DefaultRedirectUriService,
}

impl StartupOrchestrator {
    pub fn new(params: StartupOrchestratorBuilder) -> Self {
        Self {
            realm_service: params.realm_service,
            user_service: params.user_service,
            client_service: params.client_service,
            role_service: params.role_service,
            jwt_service: params.jwt_service,
            user_role_service: params.user_role_service,
            credential_service: params.credential_service,
            redirect_uri_service: params.redirect_uri_service,
        }
    }

    pub async fn initialize_application(
        &self,
        config: StartupConfig,
    ) -> Result<InitializationResult, anyhow::Error> {
        let realm = match self
            .realm_service
            .get_by_name(config.master_realm_name.clone())
            .await
        {
            Ok(realm) => {
                info!("{} already exists", config.master_realm_name);
                realm
            }
            Err(_) => {
                info!("Creating master realm");
                let realm = self
                    .realm_service
                    .create_realm(config.master_realm_name.clone())
                    .await?;
                info!("{} realm created", config.master_realm_name);
                realm
            }
        };

        self.jwt_service.retrieve_realm_rsa_keys(&realm).await?;

        let client = match self
            .client_service
            .get_by_client_id(config.default_client_id.clone(), realm.id)
            .await
        {
            Ok(client) => {
                info!(
                    "client {:} already exists",
                    config.default_client_id.clone()
                );
                client
            }
            Err(_) => {
                info!("Creating client {:}", config.default_client_id.clone());
                let client = self
                    .client_service
                    .create_client(
                        CreateClientRequest {
                            realm_id: realm.id,
                            name: config.default_client_id.clone(),
                            client_id: config.default_client_id.clone(),
                            enabled: true,
                            protocol: "openid-connect".to_string(),
                            public_client: false,
                            service_account_enabled: false,
                            client_type: "confidential".to_string(),
                            secret: Some(generate_random_string()),
                        },
                        realm.name.clone(),
                    )
                    .await
                    .map_err(|_| anyhow::anyhow!("failed to create client"))?;

                info!("client {:} created", config.default_client_id.clone());
                client
            }
        };

        let master_realm_client_id = format!("{}-realm", config.master_realm_name);

        let master_realm_client = match self
            .client_service
            .get_by_client_id(master_realm_client_id.clone(), realm.id)
            .await
        {
            Ok(client) => {
                info!("client {:} created", master_realm_client_id.clone());
                client
            }
            Err(_) => {
                info!("Creating client {:}", master_realm_client_id.clone());
                let client = self
                    .client_service
                    .create_client(
                        CreateClientRequest {
                            realm_id: realm.id,
                            name: master_realm_client_id.clone(),
                            client_id: master_realm_client_id.clone(),
                            enabled: true,
                            protocol: "openid-connect".to_string(),
                            public_client: false,
                            service_account_enabled: false,
                            client_type: "confidential".to_string(),
                            secret: Some(generate_random_string()),
                        },
                        realm.name.clone(),
                    )
                    .await
                    .map_err(|_| anyhow::anyhow!("failed to create client"))?;

                info!("client {:} created", master_realm_client_id.clone());
                client
            }
        };

        let user = match self
            .user_service
            .get_by_username(config.admin_username.clone(), realm.id)
            .await
        {
            Ok(user) => {
                let username = user.username.clone();
                info!("user {username:} already exists");
                user
            }
            Err(_) => {
                let client_id = config.default_client_id.clone();
                info!("Creating user for client {client_id:}");
                let user = self
                    .user_service
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
                    .map_err(|_| anyhow::anyhow!("failed to create user"))?;

                info!("user {:} created", user.username);
                user
            }
        };

        let roles = self
            .role_service
            .get_by_client_id(master_realm_client.id) // Updated to remove clone()
            .await
            .unwrap_or_default();
        let role = match roles
            .into_iter()
            .find(|r| r.name == master_realm_client_id.clone())
        {
            Some(role) => {
                info!("role {:} already exists", role.name);
                role
            }
            None => {
                let role = self
                    .role_service
                    .create(CreateRoleRequest {
                        client_id: Some(master_realm_client.id),
                        name: master_realm_client_id.clone(),
                        permissions: Permissions::to_names(&[Permissions::ManageRealm]),
                        realm_id: realm.id,
                        description: None,
                    })
                    .await
                    .map_err(|_| anyhow::anyhow!("failed to create role"))?;

                info!("role {:} created", master_realm_client_id.clone());
                role
            }
        };

        match self
            .user_role_service
            .assign_role(realm.name.clone(), user.id, role.id)
            .await
        {
            Ok(_) => {
                info!("role {:} assigned to user {:}", role.name, user.username);
            }
            Err(_) => {
                info!(
                    "role {:} already assigned to user {:}",
                    role.name, user.username
                );
            }
        }

        match self
            .credential_service
            .create_password_credential(user.id, config.admin_password.clone(), "".to_string())
            .await
        {
            Ok(_) => {
                info!(
                    "credential for user {:} created",
                    config.admin_username.clone()
                );
            }
            Err(_) => {
                info!(
                    "credential for user {:} already exists",
                    config.admin_username.clone()
                );
            }
        };

        let admin_redirect_patterns = vec![
            // Pattern regex pour accepter toutes les URLs sur localhost avec n'importe quel port
            "^http://localhost:[0-9]+/.*",
            "^/*",
            "http://localhost:3000/admin",
            "http://localhost:5173/admin",
        ];

        let existing_uris = self
            .redirect_uri_service
            .get_by_client_id(client.id)
            .await
            .unwrap_or_default();

        for pattern in admin_redirect_patterns {
            let pattern_exists = existing_uris.iter().any(|uri| uri.value == pattern);

            if !pattern_exists {
                let redirect_request = CreateRedirectUriRequest {
                    enabled: true,
                    value: pattern.to_string(),
                };

                match self
                    .redirect_uri_service
                    .add_redirect_uri(redirect_request, realm.name.clone(), client.id)
                    .await
                {
                    Ok(_) => info!("Successfully added admin redirect URI: {}", pattern),
                    Err(e) => info!("Failed to add admin redirect URI {}: {}", pattern, e),
                }
            } else {
                info!("Admin redirect URI already exists: {}", pattern);
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
