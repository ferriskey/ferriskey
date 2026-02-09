use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::{generate_timestamp, generate_uuid_v7};
use crate::domain::realm::entities::RealmId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClientScope {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub is_default: bool,
    pub attributes: Option<Vec<ClientScopeAttribute>>,
    pub protocol_mappers: Option<Vec<ProtocolMapper>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ClientScope {
    pub fn new(
        realm_id: RealmId,
        name: String,
        description: Option<String>,
        protocol: String,
    ) -> Self {
        let (now, timestamp) = generate_timestamp();
        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            name,
            description,
            protocol,
            is_default: false,
            attributes: None,
            protocol_mappers: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClientScopeAttribute {
    pub id: Uuid,
    pub scope_id: Uuid,
    pub name: String,
    pub value: Option<String>,
}

impl ClientScopeAttribute {
    pub fn new(scope_id: Uuid, name: String, value: Option<String>) -> Self {
        Self {
            id: generate_uuid_v7(),
            scope_id,
            name,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProtocolMapper {
    pub id: Uuid,
    pub client_scope_id: Uuid,
    pub name: String,
    pub mapper_type: String,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl ProtocolMapper {
    pub fn new(
        client_scope_id: Uuid,
        name: String,
        mapper_type: String,
        config: serde_json::Value,
    ) -> Self {
        let (now, timestamp) = generate_timestamp();
        Self {
            id: Uuid::new_v7(timestamp),
            client_scope_id,
            name,
            mapper_type,
            config,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClientScopeMapping {
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub is_default: bool,
    pub is_optional: bool,
}
