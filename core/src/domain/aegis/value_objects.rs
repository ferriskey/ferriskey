use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::realm::entities::RealmId;

// --- Repository DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientScopeRequest {
    pub realm_id: RealmId,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientScopeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub protocol: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProtocolMapperRequest {
    pub client_scope_id: Uuid,
    pub name: String,
    pub mapper_type: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProtocolMapperRequest {
    pub name: Option<String>,
    pub mapper_type: Option<String>,
    pub config: Option<serde_json::Value>,
}

// --- Service Inputs ---

pub struct CreateClientScopeInput {
    pub realm_name: String,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub is_default: bool,
}

pub struct GetClientScopeInput {
    pub realm_name: String,
    pub scope_id: Uuid,
}

pub struct GetClientScopesInput {
    pub realm_name: String,
}

pub struct UpdateClientScopeInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub payload: UpdateClientScopeRequest,
}

pub struct DeleteClientScopeInput {
    pub realm_name: String,
    pub scope_id: Uuid,
}

pub struct AssignClientScopeInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub is_default: bool,
    pub is_optional: bool,
}

pub struct UnassignClientScopeInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub scope_id: Uuid,
}

pub struct CreateProtocolMapperInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub name: String,
    pub mapper_type: String,
    pub config: serde_json::Value,
}

pub struct UpdateProtocolMapperInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub mapper_id: Uuid,
    pub payload: UpdateProtocolMapperRequest,
}

pub struct DeleteProtocolMapperInput {
    pub realm_name: String,
    pub scope_id: Uuid,
    pub mapper_id: Uuid,
}
