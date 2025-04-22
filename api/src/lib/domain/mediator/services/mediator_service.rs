use std::sync::Arc;

use tracing::info;

use crate::{
    application::http::client::validators::CreateClientValidator,
    domain::{
        client::{
            ports::client_service::ClientService, services::client_service::DefaultClientService,
        },
        credential::{
            ports::credential_service::CredentialService,
            services::credential_service::DefaultCredentialService,
        },
        realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
        user::{
            dtos::user_dto::CreateUserDto, ports::user_service::UserService,
            services::user_service::DefaultUserService,
        },
    },
};

use crate::domain::mediator::ports::mediator_service::MediatorService;

pub type DefaultMediatorService = MediatorServiceImpl;

#[derive(Debug, Clone)]
pub struct MediatorServiceImpl {
    pub client_service: Arc<DefaultClientService>,
    pub realm_service: Arc<DefaultRealmService>,
    pub user_service: Arc<DefaultUserService>,
    pub credential_service: Arc<DefaultCredentialService>,
}

impl MediatorServiceImpl {
    pub fn new(
        client_service: Arc<DefaultClientService>,
        realm_service: Arc<DefaultRealmService>,
        user_service: Arc<DefaultUserService>,
        credential_service: Arc<DefaultCredentialService>,
    ) -> Self {
        Self {
            client_service,
            realm_service,
            user_service,
            credential_service,
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

        match self.client_service.create_client(schema, realm.name).await {
            Ok(client) => {
                info!("client {:} created", client_id.clone());
                client
            }
            Err(_) => {
                info!("client {:} already exists", client_id.clone());
                self.client_service
                    .get_by_client_id(client_id, realm.id)
                    .await?
            }
        };

        let user = match self
            .user_service
            .create_user(CreateUserDto {
                email: "admin@security.com".to_string(),
                email_verified: true,
                enabled: true,
                firstname: "admin".to_string(),
                lastname: "admin".to_string(),
                realm_id: realm.id,
                client_id: None,
                username: "admin".to_string(),
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
                    .get_by_username("admin".to_string(), realm.id)
                    .await?;
                info!("user {:} already exists", "admin");
                user
            }
        };

        let _ = match self
            .credential_service
            .create_password_credential(user.id, "admin".to_string(), "".to_string())
            .await
        {
            Ok(credential) => {
                info!("credential {:} created", credential.id);
                credential
            }
            Err(_) => {
                info!("credential {:} already exists", "admin");
                return Ok(());
            }
        };

        Ok(())
    }
}
