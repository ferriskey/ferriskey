use std::sync::Arc;

use tracing::info;

use crate::{
    application::http::client::validators::CreateClientValidator,
    domain::{
        client::ports::ClientService,
        credential::ports::CredentialService,
        realm::ports::RealmService,
        user::ports::{CreateUserDto, UserService},
    },
};

use super::ports::MediatorService;

#[derive(Debug, Clone)]
pub struct MediatorServiceImpl<C, R, U, CS>
where
    C: ClientService,
    R: RealmService,
    U: UserService,
    CS: CredentialService,
{
    pub client_service: Arc<C>,
    pub realm_service: Arc<R>,
    pub user_service: Arc<U>,
    pub credential_service: Arc<CS>,
}

impl<C, R, U, CS> MediatorServiceImpl<C, R, U, CS>
where
    C: ClientService,
    R: RealmService,
    U: UserService,
    CS: CredentialService,
{
    pub fn new(
        client_service: Arc<C>,
        realm_service: Arc<R>,
        user_service: Arc<U>,
        credential_service: Arc<CS>,
    ) -> Self {
        Self {
            client_service,
            realm_service,
            user_service,
            credential_service,
        }
    }
}

impl<C, R, U, CS> MediatorService for MediatorServiceImpl<C, R, U, CS>
where
    C: ClientService,
    R: RealmService,
    U: UserService,
    CS: CredentialService,
{
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
            secret: Some("secret".to_string()),
        };

        match self.client_service.create_client(schema, realm.name).await {
            Ok(client) => {
                info!("client {:} created", client_id.clone());
                client
            }
            Err(_) => {
                info!("client {:} already exists", client_id.clone());
                self
                    .client_service
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
