use uuid::Uuid;

/// Data transfer object for creating a new client
#[derive(Debug, Clone)]
pub struct CreateClientDto {
    pub realm_id: Uuid,
    pub name: String,
    pub client_id: String,
    pub secret: String,
    pub enabled: bool,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub client_type: String,
}

/// Data transfer object for updating a client
#[derive(Debug, Clone)]
pub struct UpdateClientDto {
    pub name: Option<String>,
    pub client_id: Option<String>,
    pub enabled: Option<bool>,
}
