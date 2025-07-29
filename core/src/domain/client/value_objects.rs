use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub name: String,
    pub client_id: String,
    pub enabled: bool,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub client_id: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRedirectUriRequest {
    pub value: String,
    pub enabled: bool,
}
