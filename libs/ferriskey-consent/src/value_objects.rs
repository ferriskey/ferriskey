use uuid::Uuid;

use ferriskey_domain::realm::RealmId;

use super::entities::ScopeDescriptor;

pub const DEFAULT_CONSENT_TTL_DAYS: i64 = 30;

#[derive(Debug, Clone)]
pub struct EvaluateConsentInput {
    pub realm_id: RealmId,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub consent_required: bool,
    pub default_scopes: Vec<ScopeDescriptor>,
    pub requested_optional_scopes: Vec<ScopeDescriptor>,
    pub force_screen: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingConsentView {
    pub default_scopes: Vec<ScopeDescriptor>,
    pub optional_scopes: Vec<ScopeDescriptor>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsentEvaluation {
    Skip { granted_scopes: Vec<String> },
    Show(PendingConsentView),
}

#[derive(Debug, Clone)]
pub struct DecideConsentInput {
    pub realm_id: RealmId,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub default_scope_names: Vec<String>,
    pub requested_optional_scope_names: Vec<String>,
    pub approved_scope_names: Vec<String>,
    pub ttl_days: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsentDecisionOutcome {
    Approved { granted_scopes: Vec<String> },
    Denied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsentRequestView {
    pub client_name: String,
    pub client_uri_host: Option<String>,
    pub default_scopes: Vec<ScopeDescriptor>,
    pub optional_scopes: Vec<ScopeDescriptor>,
}
