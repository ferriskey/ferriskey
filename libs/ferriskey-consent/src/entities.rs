use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use ferriskey_domain::generate_uuid_v7;
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::RealmOwned;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ConsentDecisionId(pub Uuid);

impl ConsentDecisionId {
    pub fn new() -> Self {
        ConsentDecisionId(generate_uuid_v7())
    }
}

impl Default for ConsentDecisionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ConsentDecision {
    pub id: ConsentDecisionId,
    pub realm_id: RealmId,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub granted_scopes: Vec<String>,
    pub denied_scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConsentDecision {
    pub fn is_live(&self, now: DateTime<Utc>) -> bool {
        self.expires_at > now
    }

    pub fn covers(&self, requested_optional_scope_names: &[String]) -> bool {
        requested_optional_scope_names
            .iter()
            .all(|name| self.granted_scopes.contains(name) || self.denied_scopes.contains(name))
    }
}

impl RealmOwned for ConsentDecision {
    fn realm_id(&self) -> RealmId {
        self.realm_id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ScopeDescriptor {
    pub name: String,
    pub description: Option<String>,
}
