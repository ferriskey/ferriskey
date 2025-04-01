use std::sync::Arc;

use tracing::info;

use crate::{
    application::http::client::validators::CreateClientValidator,
    domain::{client::ports::ClientService, realm::ports::RealmService},
};

use super::ports::MediatorService;

#[derive(Debug, Clone)]
pub struct MediatorServiceImpl<C, R>
where
    C: ClientService,
    R: RealmService,
{
    pub client_service: Arc<C>,
    pub realm_service: Arc<R>,
}

impl<C, R> MediatorServiceImpl<C, R>
where
    C: ClientService,
    R: RealmService,
{
    pub fn new(client_service: Arc<C>, realm_service: Arc<R>) -> Self {
        Self {
            client_service,
            realm_service,
        }
    }
}

impl<C, R> MediatorService for MediatorServiceImpl<C, R>
where
    C: ClientService,
    R: RealmService,
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
                return Ok(());
            }
        };

        Ok(())
    }
}
