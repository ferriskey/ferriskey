use uuid::Uuid;

use super::entities::PolicyBody;
use super::names::{Namespace, PolicyName};

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
    pub body: PolicyBody,
}

pub struct UpdatePolicyInput {
    pub realm_name: String,
    pub id: Uuid,
    pub name: PolicyName,
    pub description: Option<String>,
    pub enabled: bool,
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
