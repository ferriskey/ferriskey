use serde_json::{Map, Value};
use uuid::Uuid;

use super::entities::PolicyBody;
use super::evaluation::TenantEntity;
use super::names::{ManagedBy, Namespace, PolicyName};

pub struct ReplaceSchemaInput {
    pub realm_name: String,
    pub namespace: Namespace,
    pub source: String,
}

pub struct CreatePolicyInput {
    pub realm_name: String,
    pub name: PolicyName,
    pub description: Option<String>,
    pub enabled: bool,
    pub managed_by: Option<ManagedBy>,
    pub body: PolicyBody,
}

pub struct UpdatePolicyInput {
    pub realm_name: String,
    pub id: Uuid,
    pub name: PolicyName,
    pub description: Option<String>,
    pub enabled: bool,
    pub managed_by: Option<ManagedBy>,
    pub body: PolicyBody,
}

pub struct CreateTemplateInput {
    pub realm_name: String,
    pub name: PolicyName,
    pub description: Option<String>,
    pub source: String,
}

pub struct UpdateTemplateInput {
    pub realm_name: String,
    pub id: Uuid,
    pub name: PolicyName,
    pub description: Option<String>,
    pub source: String,
}

pub struct CandidatePolicy {
    pub name: PolicyName,
    pub body: PolicyBody,
}

pub struct ValidatePoliciesInput {
    pub realm_name: String,
    pub policies: Vec<CandidatePolicy>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationRequest {
    pub principal: TenantEntity,
    pub action: String,
    pub resource: TenantEntity,
    pub context: Map<String, Value>,
    pub entities: Vec<TenantEntity>,
}

pub struct SimulateInput {
    pub realm_name: String,
    pub policies: Option<Vec<CandidatePolicy>>,
    pub request: SimulationRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationOutcome {
    pub allowed: bool,
    pub determining_policies: Vec<String>,
}
