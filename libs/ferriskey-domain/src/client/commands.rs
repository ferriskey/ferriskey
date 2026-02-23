use uuid::Uuid;

use crate::client::value_objects::UpdateClientRequest;

pub struct CreateClientInput {
    pub realm_name: String,
    pub name: String,
    pub client_id: String,
    pub client_type: String,
    pub service_account_enabled: bool,
    pub public_client: bool,
    pub protocol: String,
    pub enabled: bool,
    pub direct_access_grants_enabled: bool,
}

pub struct CreateRedirectUriInput {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: crate::client::value_objects::CreateRedirectUriRequest,
}

pub struct CreatePostLogoutRedirectUriInput {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: crate::client::value_objects::CreateRedirectUriRequest,
}

pub struct CreateRoleInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub description: Option<String>,
    pub name: String,
    pub permissions: Vec<String>,
}

pub struct DeleteClientInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct DeleteRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

pub struct DeletePostLogoutRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

pub struct GetClientInput {
    pub client_id: Uuid,
    pub realm_name: String,
}

pub struct GetClientRolesInput {
    pub client_id: Uuid,
    pub realm_name: String,
}

pub struct GetRedirectUrisInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct GetPostLogoutRedirectUrisInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct GetClientsInput {
    pub realm_name: String,
}

pub struct UpdateClientInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub payload: UpdateClientRequest,
}

pub struct UpdateRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub redirect_uri_id: Uuid,
    pub enabled: bool,
}

pub struct UpdatePostLogoutRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub redirect_uri_id: Uuid,
    pub enabled: bool,
}
