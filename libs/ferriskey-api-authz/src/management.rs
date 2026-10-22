use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ferriskey_authz::entities::{EntityRef, EntityType};
use ferriskey_authz::tenant::entities::{
    AuthzSchema, AuthzState, Policy, PolicyBody, PolicyTemplate,
};
use ferriskey_authz::tenant::evaluation::TenantEntity;
use ferriskey_authz::tenant::names::{ManagedBy, Namespace, PolicyName, SlotId};
use ferriskey_authz::tenant::value_objects::{
    CandidatePolicy, CreatePolicyInput, CreateTemplateInput, ReplaceSchemaInput, SimulateInput,
    SimulationOutcome, SimulationRequest, UpdatePolicyInput, UpdateTemplateInput,
    ValidatePoliciesInput,
};
use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::validators::{AuthzenAction, DecisionContext, EvaluationResponse, ReasonAdmin};

fn invalid(message: String) -> CoreError {
    CoreError::InvalidAuthorizationRequest(message)
}

fn name(value: String) -> Result<PolicyName, CoreError> {
    PolicyName::try_from(value)
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ManagedByQuery {
    pub managed_by: Option<String>,
}

impl ManagedByQuery {
    pub fn manager(self) -> Result<Option<ManagedBy>, CoreError> {
        self.managed_by.map(ManagedBy::try_from).transpose()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct EntityRefBody {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub id: String,
}

impl From<&EntityRef> for EntityRefBody {
    fn from(value: &EntityRef) -> Self {
        Self {
            entity_type: value.ty.as_str().to_string(),
            id: value.id.clone(),
        }
    }
}

impl From<EntityRefBody> for EntityRef {
    fn from(value: EntityRefBody) -> Self {
        Self::new(EntityType::custom(value.entity_type), value.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyBodyBody {
    Static {
        source: String,
    },
    Linked {
        template_id: Uuid,
        #[schema(value_type = HashMap<String, EntityRefBody>)]
        links: BTreeMap<SlotId, EntityRefBody>,
    },
}

impl From<PolicyBodyBody> for PolicyBody {
    fn from(value: PolicyBodyBody) -> Self {
        match value {
            PolicyBodyBody::Static { source } => Self::Static { source },
            PolicyBodyBody::Linked { template_id, links } => Self::Linked {
                template_id,
                links: links
                    .into_iter()
                    .map(|(slot, entity)| (slot, entity.into()))
                    .collect(),
            },
        }
    }
}

impl From<&PolicyBody> for PolicyBodyBody {
    fn from(value: &PolicyBody) -> Self {
        match value {
            PolicyBody::Static { source } => Self::Static {
                source: source.clone(),
            },
            PolicyBody::Linked { template_id, links } => Self::Linked {
                template_id: *template_id,
                links: links
                    .iter()
                    .map(|(slot, entity)| (*slot, entity.into()))
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct AuthzStateBody {
    pub enabled: bool,
    pub version: i64,
}

impl From<AuthzState> for AuthzStateBody {
    fn from(value: AuthzState) -> Self {
        Self {
            enabled: value.enabled,
            version: value.version,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct SetStateValidator {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct SchemaBody {
    pub namespace: String,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

impl From<AuthzSchema> for SchemaBody {
    fn from(value: AuthzSchema) -> Self {
        Self {
            namespace: value.namespace.to_string(),
            source: value.source,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct BuiltinSchemaBody {
    pub namespace: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ReplaceSchemaValidator {
    pub namespace: Option<String>,
    pub source: String,
}

impl ReplaceSchemaValidator {
    pub fn into_input(self, realm_name: String) -> Result<ReplaceSchemaInput, CoreError> {
        Ok(ReplaceSchemaInput {
            realm_name,
            namespace: self
                .namespace
                .map(Namespace::try_from)
                .transpose()?
                .unwrap_or_default(),
            source: self.source,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct PolicyResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub origin: String,
    pub enabled: bool,
    pub managed_by: Option<String>,
    pub body: PolicyBodyBody,
}

impl From<Policy> for PolicyResponse {
    fn from(value: Policy) -> Self {
        Self {
            id: value.id,
            name: value.name.to_string(),
            description: value.description,
            origin: match value.origin {
                ferriskey_authz::entities::PolicyOrigin::Builtin => "builtin",
                ferriskey_authz::entities::PolicyOrigin::Generated => "generated",
                ferriskey_authz::entities::PolicyOrigin::Custom => "custom",
            }
            .to_string(),
            enabled: value.enabled,
            managed_by: value.managed_by.map(|m| m.to_string()),
            body: PolicyBodyBody::from(&value.body),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct PolicyValidator {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub body: PolicyBodyBody,
}

impl PolicyValidator {
    pub fn into_create(
        self,
        realm_name: String,
        managed_by: Option<ManagedBy>,
    ) -> Result<CreatePolicyInput, CoreError> {
        Ok(CreatePolicyInput {
            realm_name,
            name: name(self.name)?,
            description: self.description,
            enabled: self.enabled,
            managed_by,
            body: self.body.into(),
        })
    }

    pub fn into_update(
        self,
        realm_name: String,
        id: Uuid,
        managed_by: Option<ManagedBy>,
    ) -> Result<UpdatePolicyInput, CoreError> {
        Ok(UpdatePolicyInput {
            realm_name,
            id,
            name: name(self.name)?,
            description: self.description,
            enabled: self.enabled,
            managed_by,
            body: self.body.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
}

impl From<PolicyTemplate> for TemplateResponse {
    fn from(value: PolicyTemplate) -> Self {
        Self {
            id: value.id,
            name: value.name.to_string(),
            description: value.description,
            source: value.source,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct TemplateValidator {
    pub name: String,
    pub description: Option<String>,
    pub source: String,
}

impl TemplateValidator {
    pub fn into_create(self, realm_name: String) -> Result<CreateTemplateInput, CoreError> {
        Ok(CreateTemplateInput {
            realm_name,
            name: name(self.name)?,
            description: self.description,
            source: self.source,
        })
    }

    pub fn into_update(
        self,
        realm_name: String,
        id: Uuid,
    ) -> Result<UpdateTemplateInput, CoreError> {
        Ok(UpdateTemplateInput {
            realm_name,
            id,
            name: name(self.name)?,
            description: self.description,
            source: self.source,
        })
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CandidateValidator {
    pub name: String,
    pub body: PolicyBodyBody,
}

impl TryFrom<CandidateValidator> for CandidatePolicy {
    type Error = CoreError;

    fn try_from(value: CandidateValidator) -> Result<Self, Self::Error> {
        Ok(Self {
            name: name(value.name)?,
            body: value.body.into(),
        })
    }
}

fn candidates(values: Vec<CandidateValidator>) -> Result<Vec<CandidatePolicy>, CoreError> {
    values.into_iter().map(CandidatePolicy::try_from).collect()
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ValidatePoliciesValidator {
    pub policies: Vec<CandidateValidator>,
}

impl ValidatePoliciesValidator {
    pub fn into_input(self, realm_name: String) -> Result<ValidatePoliciesInput, CoreError> {
        Ok(ValidatePoliciesInput {
            realm_name,
            policies: candidates(self.policies)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct IssueBody {
    pub policy: Option<String>,
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl From<PolicyIssue> for IssueBody {
    fn from(value: PolicyIssue) -> Self {
        Self {
            policy: value.policy,
            message: value.message,
            line: value.line,
            column: value.column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct ValidationResponse {
    pub valid: bool,
    pub issues: Vec<IssueBody>,
}

impl From<Vec<PolicyIssue>> for ValidationResponse {
    fn from(value: Vec<PolicyIssue>) -> Self {
        Self {
            valid: value.is_empty(),
            issues: value.into_iter().map(IssueBody::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SimulatedEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub id: String,
    #[serde(default)]
    pub properties: Option<Map<String, Value>>,
    #[serde(default)]
    pub parents: Option<Vec<EntityRefBody>>,
}

impl TryFrom<SimulatedEntity> for TenantEntity {
    type Error = CoreError;

    fn try_from(value: SimulatedEntity) -> Result<Self, Self::Error> {
        if value.entity_type.is_empty() || value.id.is_empty() {
            return Err(invalid("entity type and id must not be empty".to_string()));
        }
        Ok(Self {
            entity: EntityRef::new(EntityType::custom(value.entity_type), value.id),
            properties: value.properties.unwrap_or_default(),
            parents: value
                .parents
                .unwrap_or_default()
                .into_iter()
                .map(EntityRef::from)
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SimulatedRequest {
    pub subject: SimulatedEntity,
    pub action: AuthzenAction,
    pub resource: SimulatedEntity,
    #[serde(default)]
    pub context: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct SimulateValidator {
    pub policies: Option<Vec<CandidateValidator>>,
    #[serde(default)]
    pub entities: Vec<SimulatedEntity>,
    pub request: SimulatedRequest,
}

impl SimulateValidator {
    pub fn into_input(self, realm_name: String) -> Result<SimulateInput, CoreError> {
        if self.request.action.name.is_empty() {
            return Err(invalid("action.name must not be empty".to_string()));
        }
        Ok(SimulateInput {
            realm_name,
            policies: self.policies.map(candidates).transpose()?,
            request: SimulationRequest {
                principal: self.request.subject.try_into()?,
                action: self.request.action.name,
                resource: self.request.resource.try_into()?,
                context: self.request.context.unwrap_or_default(),
                entities: self
                    .entities
                    .into_iter()
                    .map(TenantEntity::try_from)
                    .collect::<Result<_, _>>()?,
            },
        })
    }
}

impl From<SimulationOutcome> for EvaluationResponse {
    fn from(value: SimulationOutcome) -> Self {
        Self {
            decision: value.allowed,
            context: Some(DecisionContext {
                reason_admin: ReasonAdmin {
                    policies: value.determining_policies,
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_static_and_a_linked_body_round_trip() {
        let static_body: PolicyBodyBody = serde_json::from_value(
            json!({"type": "static", "source": "permit(principal, action, resource);"}),
        )
        .expect("static");
        let linked: PolicyBodyBody = serde_json::from_value(json!({
            "type": "linked",
            "template_id": Uuid::nil(),
            "links": {"?principal": {"type": "FerrisKey::Role", "id": "r1"}},
        }))
        .expect("linked");

        for body in [static_body, linked] {
            let domain = PolicyBody::from(body.clone());
            assert_eq!(PolicyBodyBody::from(&domain), body);
        }
    }

    #[test]
    fn an_unknown_slot_is_rejected() {
        assert!(
            serde_json::from_value::<PolicyBodyBody>(json!({
                "type": "linked",
                "template_id": Uuid::nil(),
                "links": {"?other": {"type": "FerrisKey::Role", "id": "r1"}},
            }))
            .is_err()
        );
    }

    #[test]
    fn a_simulation_keeps_parents_and_rejects_empty_actions() {
        let ok: SimulateValidator = serde_json::from_value(json!({
            "entities": [{"type": "FerrisKey::Group", "id": "g1"}],
            "request": {
                "subject": {"type": "FerrisKey::User", "id": "u1", "parents": [{"type": "FerrisKey::Group", "id": "g1"}]},
                "action": {"name": "call_tool"},
                "resource": {"type": "FerrisKey::Tool", "id": "files/read"},
            },
        }))
        .expect("parse");
        let input = ok.into_input("acme".to_string()).expect("input");
        assert_eq!(input.request.principal.parents.len(), 1);
        assert_eq!(input.request.entities.len(), 1);

        let empty: SimulateValidator = serde_json::from_value(json!({
            "request": {
                "subject": {"type": "FerrisKey::User", "id": "u1"},
                "action": {"name": ""},
                "resource": {"type": "FerrisKey::Tool", "id": "t"},
            },
        }))
        .expect("parse");
        assert!(empty.into_input("acme".to_string()).is_err());
    }

    #[test]
    fn a_manager_label_is_validated() {
        assert!(
            ManagedByQuery {
                managed_by: Some("Bad Label".to_string())
            }
            .manager()
            .is_err()
        );
        assert!(
            ManagedByQuery { managed_by: None }
                .manager()
                .expect("none")
                .is_none()
        );
    }
}
