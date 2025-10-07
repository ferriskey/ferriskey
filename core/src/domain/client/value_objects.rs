use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub realm_id: Uuid,
    pub name: String,
    pub client_id: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub direct_access_grants_enabled: bool,
    pub client_type: String,
}

impl CreateClientRequest {
    pub fn create_realm_system_client(realm_id: Uuid, client_name: String) -> CreateClientRequest {
        CreateClientRequest {
            realm_id,
            client_id: client_name.clone(),
            client_type: "system".to_string(),
            direct_access_grants_enabled: false,
            enabled: true,
            name: client_name,
            protocol: "openid-connect".to_string(),
            public_client: true,
            secret: None,
            service_account_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub client_id: Option<String>,
    pub enabled: Option<bool>,
    pub direct_access_grants_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRedirectUriRequest {
    pub value: String,
    pub enabled: bool,
}
