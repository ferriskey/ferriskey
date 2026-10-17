use std::collections::BTreeMap;

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::scope::RealmScope;
use serde_json::{Map, Value};
use uuid::Uuid;

use super::entities::{AuthzSchema, Policy, PolicyTemplate};
use super::ports::{AuthzSchemaRepository, PolicyRepository, PolicyTemplateRepository};
use crate::entities::EntityRef;

#[derive(Clone, Debug, PartialEq)]
pub struct RuleSet {
    pub version: i64,
    pub schema: Option<AuthzSchema>,
    pub policies: Vec<Policy>,
    pub templates: Vec<PolicyTemplate>,
}

impl RuleSet {
    pub fn enabled_policies(&self) -> impl Iterator<Item = &Policy> {
        self.policies.iter().filter(|p| p.enabled)
    }
}

pub(crate) async fn load_rule_set<SC, PR, T>(
    scope: &RealmScope,
    version: i64,
    schemas: &SC,
    policies: &PR,
    templates: &T,
) -> Result<RuleSet, CoreError>
where
    SC: AuthzSchemaRepository,
    PR: PolicyRepository,
    T: PolicyTemplateRepository,
{
    Ok(RuleSet {
        version,
        schema: schemas.get(scope).await?,
        policies: policies.list(scope).await?,
        templates: templates.list(scope).await?,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct TenantEntity {
    pub entity: EntityRef,
    pub properties: Map<String, Value>,
    pub parents: Vec<EntityRef>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Subject {
    FerrisKey(EntityRef),
    Tenant(TenantEntity),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationRequest {
    pub subject: Subject,
    pub action: String,
    pub resource: TenantEntity,
    pub context: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserFacts {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
    pub tags: BTreeMap<String, String>,
    pub role_ids: Vec<Uuid>,
    pub group_ids: Vec<Uuid>,
    pub organization_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientFacts {
    pub id: Uuid,
    pub client_id: String,
    pub role_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrincipalFacts {
    User(UserFacts),
    Client(ClientFacts),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoleFacts {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupFacts {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub organization_id: Uuid,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrganizationFacts {
    pub id: Uuid,
    pub alias: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrincipalSlice {
    pub principal: PrincipalFacts,
    pub roles: Vec<RoleFacts>,
    pub groups: Vec<GroupFacts>,
    pub organizations: Vec<OrganizationFacts>,
}
