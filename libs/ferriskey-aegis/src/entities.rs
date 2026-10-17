use std::collections::HashSet;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use ferriskey_domain::common::pagination::DateRange;
use ferriskey_domain::realm::scope::RealmOwned;
use ferriskey_domain::{generate_timestamp, generate_uuid_v7, realm::RealmId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;
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
            other => {
                warn!("Unknown ScopeType value: '{}', defaulting to None", other);
                ScopeType::None
            }
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
    pub dynamic_registration_allowed: bool,
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
            dynamic_registration_allowed: false,
            attributes: None,
            protocol_mappers: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl RealmOwned for ClientScope {
    fn realm_id(&self) -> RealmId {
        self.realm_id
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientScopeFilter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub search: Option<String>,
    pub protocol: Option<String>,
    pub default_scope_type: Option<ScopeType>,
    pub has_protocol_mappers: Option<bool>,
    pub not_assigned_to_client: Option<Uuid>,
    pub created: DateRange,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClientScopeSortField {
    Name,
    #[default]
    CreatedAt,
    UpdatedAt,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientScopeSet {
    pub default: Vec<ClientScope>,
    pub optional: Vec<ClientScope>,
}

impl ClientScopeSet {
    pub fn applicable(
        &self,
        token_scope: Option<&str>,
        only_token_scopes: bool,
    ) -> Vec<&ClientScope> {
        let requested: HashSet<&str> = token_scope
            .map(|scope| scope.split_whitespace().collect())
            .unwrap_or_default();

        self.default
            .iter()
            .filter(|scope| !only_token_scopes || requested.contains(scope.name.as_str()))
            .chain(
                self.optional
                    .iter()
                    .filter(|scope| requested.contains(scope.name.as_str())),
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(name: &str) -> ClientScope {
        ClientScope::new(
            RealmId::from(Uuid::new_v4()),
            name.to_string(),
            None,
            "openid-connect".to_string(),
        )
    }

    fn names<'a>(scopes: &[&'a ClientScope]) -> Vec<&'a str> {
        scopes.iter().map(|scope| scope.name.as_str()).collect()
    }

    fn set() -> ClientScopeSet {
        ClientScopeSet {
            default: vec![scope("profile"), scope("email")],
            optional: vec![scope("groups"), scope("billing")],
        }
    }

    #[test]
    fn default_scopes_always_apply_and_optional_ones_only_when_the_token_names_them() {
        let set = set();

        assert_eq!(
            names(&set.applicable(Some("openid groups"), false)),
            ["profile", "email", "groups"]
        );
        assert_eq!(names(&set.applicable(None, false)), ["profile", "email"]);
    }

    #[test]
    fn only_token_scopes_narrows_the_defaults_to_the_token() {
        let set = set();

        assert_eq!(
            names(&set.applicable(Some("email groups"), true)),
            ["email", "groups"]
        );
        assert!(set.applicable(None, true).is_empty());
    }
}
