use ferriskey_domain::realm::RealmId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Repository DTOs ---

/// DTO for creating a new client scope in the repository layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientScopeRequest {
    pub realm_id: RealmId,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub is_default: bool,
}

/// DTO for updating an existing client scope in the repository layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientScopeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub protocol: Option<String>,
    pub is_default: Option<bool>,
}

/// DTO for creating a new protocol mapper in the repository layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProtocolMapperRequest {
    pub client_scope_id: Uuid,
    pub name: String,
    pub mapper_type: String,
    pub config: serde_json::Value,
}

/// DTO for updating an existing protocol mapper in the repository layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProtocolMapperRequest {
    pub name: Option<String>,
    pub mapper_type: Option<String>,
    pub config: Option<serde_json::Value>,
}

// --- Service Inputs ---

/// Input struct for creating a client scope, containing the realm name, scope name, description, protocol, and default status.
pub struct CreateClientScopeInput {
    pub realm_name: String,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub is_default: bool,
}

/// Input struct for retrieving a specific client scope, containing the realm name and scope ID.
pub struct GetClientScopeInput {
    pub realm_name: String,
    pub scope_id: Uuid,
}

/// Input struct for retrieving all client scopes within a realm, containing only the realm name.
pub struct GetClientScopesInput {
    pub realm_name: String,
}

/// Input struct for updating a client scope, containing the realm name, scope ID, and the update payload.
pub struct UpdateClientScopeInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub payload: UpdateClientScopeRequest,
}

/// Input struct for deleting a client scope, containing the realm name and scope ID.
pub struct DeleteClientScopeInput {
    pub realm_name: String,
    pub scope_id: Uuid,
}

/// Input struct for assigning a client scope to a specific client, containing the realm name, client ID, scope ID, and flags for default and optional status.
pub struct AssignClientScopeInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub is_default: bool,
    pub is_optional: bool,
}

/// Input struct for unassigning a client scope from a specific client, containing the realm name, client ID, and scope ID.
pub struct UnassignClientScopeInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub scope_id: Uuid,
}

/// Input struct for retrieving client scopes assigned to a specific client, containing the realm name and client ID.
pub struct GetClientClientScopesInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

/// Input struct for creating a protocol mapper, containing the realm name, scope ID, mapper name, type, and configuration.
pub struct CreateProtocolMapperInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub name: String,
    pub mapper_type: String,
    pub config: serde_json::Value,
}

/// Input struct for updating a protocol mapper, containing the realm name, scope ID, mapper ID, and the update payload.
pub struct UpdateProtocolMapperInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub mapper_id: Uuid,
    pub payload: UpdateProtocolMapperRequest,
}

/// Input struct for deleting a protocol mapper, containing the realm name, scope ID, and mapper ID.
pub struct DeleteProtocolMapperInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub mapper_id: Uuid,
}
