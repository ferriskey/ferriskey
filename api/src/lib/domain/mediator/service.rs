use std::sync::Arc;

use tracing::info;

use crate::{
    application::http::client::validators::CreateClientValidator,
    domain::{
        client::ports::ClientService,
        realm::ports::RealmService,
        user::{
            entities::model::UserConfig,
            ports::{CreateUserDto, UserService},
        },
    },
};

use super::ports::MediatorService;

#[derive(Debug, Clone)]
pub struct MediatorServiceImpl<C, R, U>
where
    C: ClientService,
    R: RealmService,
    U: UserService,
{
    pub client_service: Arc<C>,
    pub realm_service: Arc<R>,
    pub user_service: Arc<U>,
}

impl<C, R, U> MediatorServiceImpl<C, R, U>
where
    C: ClientService,
    R: RealmService,
    U: UserService,
{
    pub fn new(client_service: Arc<C>, realm_service: Arc<R>, user_service: Arc<U>) -> Self {
        Self {
            client_service,
            realm_service,
            user_service,
        }
    }
}

impl<C, R, U> MediatorService for MediatorServiceImpl<C, R, U>
where
    C: ClientService,
    R: RealmService,
    U: UserService,
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
                let client = self
                    .client_service
                    .get_by_client_id(client_id, realm.id)
                    .await?;
                client
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
                info!("user {:} already exists", "admin");
                return Ok(());
            }
        };

        Ok(())
    }
}
