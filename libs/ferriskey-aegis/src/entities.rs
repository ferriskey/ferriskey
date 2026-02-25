use std::fmt::Display;

use chrono::{DateTime, Utc};
use ferriskey_domain::{generate_timestamp, generate_uuid_v7, realm::RealmId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScopeType {
    #[default]
    None,
    Optional,
    Default,
}

impl From<String> for ScopeType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "OPTIONAL" => ScopeType::Optional,
            "DEFAULT" => ScopeType::Default,
            _ => ScopeType::None,
        }
    }
}

impl From<ScopeType> for String {
    fn from(value: ScopeType) -> Self {
        match value {
            ScopeType::None => "NONE".to_string(),
            ScopeType::Optional => "OPTIONAL".to_string(),
            ScopeType::Default => "DEFAULT".to_string(),
        }
    }
}

impl Display for ScopeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ScopeType {
    pub fn as_str(&self) -> &str {
        match self {
            ScopeType::None => "NONE",
            ScopeType::Optional => "OPTIONAL",
            ScopeType::Default => "DEFAULT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClientScope {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub default_scope_type: ScopeType,
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
            default_scope_type: ScopeType::None,
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
    pub config: Value,
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
    pub default_scope_type: ScopeType,
}
