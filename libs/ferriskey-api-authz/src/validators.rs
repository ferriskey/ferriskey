use ferriskey_authz::entities::{EntityRef, EntityType};
use ferriskey_authz::tenant::FERRISKEY_NAMESPACE;
use ferriskey_authz::tenant::evaluation::{
    EvaluationRequest, EvaluationsSemantic, Subject, TenantEntity,
};
use ferriskey_authz::tenant::ports::EvaluationOutcome;
use ferriskey_domain::common::app_errors::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthzenEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub id: String,
    #[serde(default)]
    pub properties: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthzenAction {
    pub name: String,
    #[serde(default)]
    pub properties: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct EvaluationValidator {
    pub subject: AuthzenEntity,
    pub action: AuthzenAction,
    pub resource: AuthzenEntity,
    #[serde(default)]
    pub context: Option<Map<String, Value>>,
}

fn required(field: &str, value: &str) -> Result<(), CoreError> {
    if value.is_empty() {
        return Err(CoreError::InvalidAuthorizationRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn ferriskey(ty: &str) -> EntityType {
    EntityType::custom(format!("{FERRISKEY_NAMESPACE}::{ty}"))
}

fn strings(properties: &Map<String, Value>, key: &str) -> Vec<String> {
    properties
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn text(properties: &Map<String, Value>, key: &str) -> Option<String> {
    properties
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn agent_user(id: String, properties: Map<String, Value>) -> TenantEntity {
    let roles = strings(&properties, "roles");
    let groups = strings(&properties, "groups");
    let organisation = text(&properties, "organisation");

    let mut attributes = Map::new();
    attributes.insert(
        "username".to_string(),
        Value::String(text(&properties, "username").unwrap_or_else(|| id.clone())),
    );
    if let Some(email) = text(&properties, "email") {
        attributes.insert("email".to_string(), Value::String(email));
    }
    attributes.insert(
        "email_verified".to_string(),
        properties
            .get("email_verified")
            .filter(|value| value.is_boolean())
            .cloned()
            .unwrap_or(Value::Bool(false)),
    );
    attributes.insert(
        "enabled".to_string(),
        properties
            .get("enabled")
            .filter(|value| value.is_boolean())
            .cloned()
            .unwrap_or(Value::Bool(true)),
    );
    attributes.insert(
        "roles".to_string(),
        Value::Array(roles.iter().cloned().map(Value::String).collect()),
    );
    if let Some(organisation) = &organisation {
        attributes.insert(
            "organisation".to_string(),
            Value::String(organisation.clone()),
        );
    }

    let mut parents: Vec<EntityRef> = groups
        .into_iter()
        .map(|group| EntityRef::new(ferriskey("Group"), group))
        .collect();
    parents.extend(
        roles
            .into_iter()
            .map(|role| EntityRef::new(ferriskey("Role"), role)),
    );
    parents.extend(
        organisation.map(|organisation| EntityRef::new(ferriskey("Organization"), organisation)),
    );

    TenantEntity {
        entity: EntityRef::new(ferriskey("User"), id),
        properties: attributes,
        parents,
    }
}

fn agent_client(id: String, properties: &Map<String, Value>) -> TenantEntity {
    let mut attributes = Map::new();
    attributes.insert("client_id".to_string(), Value::String(id.clone()));
    TenantEntity {
        entity: EntityRef::new(ferriskey("Agent"), id),
        properties: attributes,
        parents: strings(properties, "roles")
            .into_iter()
            .map(|role| EntityRef::new(ferriskey("Role"), role))
            .collect(),
    }
}

fn mcp_server_of(id: &str, properties: &Map<String, Value>) -> String {
    text(properties, "server").unwrap_or_else(|| {
        id.split_once('/')
            .map_or(id, |(server, _)| server)
            .to_string()
    })
}

fn agent_resource(kind: &str, id: String, properties: &Map<String, Value>) -> TenantEntity {
    let server = mcp_server_of(&id, properties);
    let leaf = id
        .split_once('/')
        .map_or(id.as_str(), |(_, rest)| rest)
        .to_string();
    let mut attributes = Map::new();
    let parents = match kind {
        "mcp_server" | "mcpserver" => {
            attributes.insert(
                "name".to_string(),
                Value::String(text(properties, "name").unwrap_or_else(|| id.clone())),
            );
            Vec::new()
        }
        other => {
            attributes.insert("server".to_string(), Value::String(server.clone()));
            match other {
                "tool" => {
                    attributes.insert(
                        "tool".to_string(),
                        Value::String(text(properties, "tool").unwrap_or(leaf)),
                    );
                    if let Some(annotations) = properties.get("annotations") {
                        attributes.insert("annotations".to_string(), annotations.clone());
                    }
                }
                "resource" => {
                    attributes.insert(
                        "uri".to_string(),
                        Value::String(text(properties, "uri").unwrap_or_else(|| id.clone())),
                    );
                }
                _ => {
                    attributes.insert(
                        "name".to_string(),
                        Value::String(text(properties, "name").unwrap_or(leaf)),
                    );
                }
            }
            vec![EntityRef::new(ferriskey("McpServer"), server)]
        }
    };
    let ty = match kind {
        "tool" => "Tool",
        "resource" => "Resource",
        "prompt" => "Prompt",
        _ => "McpServer",
    };
    TenantEntity {
        entity: EntityRef::new(ferriskey(ty), id),
        properties: attributes,
        parents,
    }
}

impl TryFrom<AuthzenEntity> for TenantEntity {
    type Error = CoreError;

    fn try_from(value: AuthzenEntity) -> Result<Self, Self::Error> {
        required("type", &value.entity_type)?;
        required("id", &value.id)?;
        let kind = value.entity_type.to_ascii_lowercase();
        if matches!(
            kind.as_str(),
            "tool" | "resource" | "prompt" | "mcp_server" | "mcpserver"
        ) {
            let properties = value.properties.unwrap_or_default();
            return Ok(agent_resource(&kind, value.id, &properties));
        }
        Ok(Self {
            entity: EntityRef::new(EntityType::custom(value.entity_type), value.id),
            properties: value.properties.unwrap_or_default(),
            parents: Vec::new(),
        })
    }
}

impl TryFrom<AuthzenEntity> for Subject {
    type Error = CoreError;

    fn try_from(value: AuthzenEntity) -> Result<Self, Self::Error> {
        required("type", &value.entity_type)?;
        required("id", &value.id)?;
        match value.entity_type.to_ascii_lowercase().as_str() {
            "user" => {
                return Ok(Self::Asserted(agent_user(
                    value.id,
                    value.properties.unwrap_or_default(),
                )));
            }
            "agent" | "client" => {
                return Ok(Self::Asserted(agent_client(
                    value.id,
                    &value.properties.unwrap_or_default(),
                )));
            }
            _ => {}
        }
        let reserved = format!("{FERRISKEY_NAMESPACE}::");
        if value.entity_type.starts_with(&reserved) {
            return Ok(Self::FerrisKey(EntityRef::new(
                EntityType::custom(value.entity_type),
                value.id,
            )));
        }
        TenantEntity::try_from(value).map(Self::Tenant)
    }
}

impl TryFrom<EvaluationValidator> for EvaluationRequest {
    type Error = CoreError;

    fn try_from(value: EvaluationValidator) -> Result<Self, Self::Error> {
        required("action.name", &value.action.name)?;
        Ok(Self {
            subject: Subject::try_from(value.subject)?,
            action: value.action.name,
            resource: TenantEntity::try_from(value.resource)?,
            context: value.context.unwrap_or_default(),
        })
    }
}

pub const MAX_BATCH: usize = 100;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationsSemanticBody {
    #[default]
    ExecuteAll,
    DenyOnFirstDeny,
    PermitOnFirstPermit,
}

impl From<EvaluationsSemanticBody> for EvaluationsSemantic {
    fn from(value: EvaluationsSemanticBody) -> Self {
        match value {
            EvaluationsSemanticBody::ExecuteAll => Self::ExecuteAll,
            EvaluationsSemanticBody::DenyOnFirstDeny => Self::DenyOnFirstDeny,
            EvaluationsSemanticBody::PermitOnFirstPermit => Self::PermitOnFirstPermit,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct EvaluationsOptions {
    #[serde(default)]
    pub evaluations_semantic: Option<EvaluationsSemanticBody>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct EvaluationItem {
    #[serde(default)]
    pub subject: Option<AuthzenEntity>,
    #[serde(default)]
    pub action: Option<AuthzenAction>,
    #[serde(default)]
    pub resource: Option<AuthzenEntity>,
    #[serde(default)]
    pub context: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct EvaluationsValidator {
    #[serde(flatten)]
    pub defaults: EvaluationItem,
    #[serde(default)]
    pub evaluations: Option<Vec<EvaluationItem>>,
    #[serde(default)]
    pub options: Option<EvaluationsOptions>,
}

fn present<T>(field: &str, value: Option<T>) -> Result<T, CoreError> {
    value.ok_or_else(|| {
        CoreError::InvalidAuthorizationRequest(format!(
            "{field} is required, at the top level or on every evaluation"
        ))
    })
}

impl EvaluationsValidator {
    pub fn semantic(&self) -> EvaluationsSemantic {
        self.options
            .as_ref()
            .and_then(|options| options.evaluations_semantic)
            .unwrap_or_default()
            .into()
    }

    pub fn requests(&self) -> Result<Option<Vec<EvaluationRequest>>, CoreError> {
        let Some(items) = &self.evaluations else {
            return Ok(None);
        };
        if items.is_empty() || items.len() > MAX_BATCH {
            return Err(CoreError::InvalidAuthorizationRequest(format!(
                "evaluations must hold between 1 and {MAX_BATCH} items"
            )));
        }
        items
            .iter()
            .map(|item| {
                EvaluationRequest::try_from(EvaluationValidator {
                    subject: present(
                        "subject",
                        item.subject
                            .clone()
                            .or_else(|| self.defaults.subject.clone()),
                    )?,
                    action: present(
                        "action",
                        item.action.clone().or_else(|| self.defaults.action.clone()),
                    )?,
                    resource: present(
                        "resource",
                        item.resource
                            .clone()
                            .or_else(|| self.defaults.resource.clone()),
                    )?,
                    context: item
                        .context
                        .clone()
                        .or_else(|| self.defaults.context.clone()),
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub fn single(&self) -> Result<EvaluationRequest, CoreError> {
        EvaluationRequest::try_from(EvaluationValidator {
            subject: present("subject", self.defaults.subject.clone())?,
            action: present("action", self.defaults.action.clone())?,
            resource: present("resource", self.defaults.resource.clone())?,
            context: self.defaults.context.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct EvaluationError {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct BatchDecisionContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_admin: Option<ReasonAdmin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<EvaluationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct BatchDecision {
    pub decision: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<BatchDecisionContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct EvaluationsResponse {
    pub evaluations: Vec<BatchDecision>,
}

impl From<Result<EvaluationOutcome, CoreError>> for BatchDecision {
    fn from(value: Result<EvaluationOutcome, CoreError>) -> Self {
        match value {
            Ok(outcome) => Self {
                decision: outcome.allowed,
                context: outcome.reason_admin.map(|policies| BatchDecisionContext {
                    reason_admin: Some(ReasonAdmin { policies }),
                    error: None,
                }),
            },
            Err(error) => Self {
                decision: false,
                context: Some(BatchDecisionContext {
                    reason_admin: None,
                    error: Some(EvaluationError {
                        status: 400,
                        message: error.to_string(),
                    }),
                }),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct ReasonAdmin {
    pub policies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct DecisionContext {
    pub reason_admin: ReasonAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct EvaluationResponse {
    pub decision: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<DecisionContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct AuthzenConfiguration {
    pub policy_decision_point: String,
    pub access_evaluation_endpoint: String,
    pub access_evaluations_endpoint: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(value: Value) -> Result<EvaluationRequest, CoreError> {
        let validator: EvaluationValidator = serde_json::from_value(value)
            .map_err(|e| CoreError::InvalidAuthorizationRequest(e.to_string()))?;
        EvaluationRequest::try_from(validator)
    }

    fn batch(value: Value) -> EvaluationsValidator {
        serde_json::from_value(value).expect("batch")
    }

    #[test]
    fn top_level_fields_are_defaults_an_item_may_override() {
        let validator = batch(json!({
            "subject": {"type": "FerrisKey::User", "id": "u"},
            "action": {"name": "read"},
            "context": {"ip": "10.0.0.1"},
            "evaluations": [
                {"resource": {"type": "App::Document", "id": "1"}},
                {
                    "resource": {"type": "App::Document", "id": "2"},
                    "action": {"name": "write"},
                    "context": {"ip": "10.0.0.2"},
                },
            ],
            "options": {"evaluations_semantic": "deny_on_first_deny"},
        }));

        let requests = validator.requests().expect("requests").expect("batch");

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].action, "read");
        assert_eq!(requests[1].action, "write");
        assert_eq!(requests[0].context["ip"], json!("10.0.0.1"));
        assert_eq!(requests[1].context["ip"], json!("10.0.0.2"));
        assert_eq!(validator.semantic(), EvaluationsSemantic::DenyOnFirstDeny);
    }

    #[test]
    fn a_batch_missing_a_required_field_is_refused() {
        let validator = batch(json!({
            "subject": {"type": "FerrisKey::User", "id": "u"},
            "evaluations": [{"resource": {"type": "App::Document", "id": "1"}}],
        }));
        assert!(validator.requests().is_err());
    }

    #[test]
    fn a_batch_has_between_one_and_the_maximum_items() {
        let empty = batch(json!({"evaluations": []}));
        assert!(empty.requests().is_err());

        let item = json!({
            "subject": {"type": "FerrisKey::User", "id": "u"},
            "action": {"name": "read"},
            "resource": {"type": "App::Document", "id": "1"},
        });
        let too_many = batch(json!({"evaluations": vec![item.clone(); MAX_BATCH + 1]}));
        assert!(too_many.requests().is_err());
        let enough = batch(json!({"evaluations": vec![item; MAX_BATCH]}));
        assert_eq!(
            enough.requests().expect("ok").map(|r| r.len()),
            Some(MAX_BATCH)
        );
    }

    #[test]
    fn without_evaluations_the_request_is_a_single_evaluation() {
        let validator = batch(json!({
            "subject": {"type": "FerrisKey::User", "id": "u"},
            "action": {"name": "read"},
            "resource": {"type": "App::Document", "id": "1"},
        }));
        assert!(validator.requests().expect("ok").is_none());
        assert_eq!(validator.single().expect("single").action, "read");
        assert_eq!(validator.semantic(), EvaluationsSemantic::ExecuteAll);
    }

    #[test]
    fn an_unknown_semantic_is_refused() {
        assert!(
            serde_json::from_value::<EvaluationsValidator>(
                json!({"options": {"evaluations_semantic": "first_wins"}})
            )
            .is_err()
        );
    }

    #[test]
    fn a_failed_item_is_a_deny_that_says_why() {
        let decision = BatchDecision::from(Err(CoreError::InvalidAuthorizationRequest(
            "unknown action".to_string(),
        )));
        assert!(!decision.decision);
        let error = decision.context.and_then(|c| c.error).expect("error");
        assert_eq!(error.status, 400);
    }

    #[test]
    fn a_user_subject_becomes_a_described_ferriskey_user_with_its_groups_as_parents() {
        let request = parse(json!({
            "subject": {
                "type": "user",
                "id": "u-1",
                "properties": {"roles": ["dev"], "groups": ["support"], "organisation": "acme"},
            },
            "action": {"name": "call_tool"},
            "resource": {
                "type": "tool",
                "id": "files/read",
                "properties": {"annotations": {"readOnlyHint": true}},
            },
            "context": {"agent": "claude"},
        }))
        .expect("valid");

        let Subject::Asserted(user) = request.subject else {
            panic!("expected an asserted subject");
        };
        assert_eq!(user.entity.ty.as_str(), "FerrisKey::User");
        assert_eq!(user.properties["roles"], json!(["dev"]));
        assert_eq!(user.properties["organisation"], json!("acme"));
        let parents: Vec<String> = user
            .parents
            .iter()
            .map(|p| format!("{}::{}", p.ty, p.id))
            .collect();
        assert_eq!(
            parents,
            [
                "FerrisKey::Group::support",
                "FerrisKey::Role::dev",
                "FerrisKey::Organization::acme"
            ]
        );

        assert_eq!(request.resource.entity.ty.as_str(), "FerrisKey::Tool");
        assert_eq!(request.resource.properties["server"], json!("files"));
        assert_eq!(request.resource.properties["tool"], json!("read"));
        assert_eq!(
            request.resource.properties["annotations"],
            json!({"readOnlyHint": true})
        );
        assert_eq!(request.resource.parents[0].id, "files");
    }

    #[test]
    fn a_client_or_agent_subject_becomes_a_ferriskey_agent() {
        for ty in ["client", "agent", "Agent"] {
            let request = parse(json!({
                "subject": {"type": ty, "id": "claude"},
                "action": {"name": "call_tool"},
                "resource": {"type": "prompt", "id": "docs/summarise"},
            }))
            .expect("valid");
            let Subject::Asserted(agent) = request.subject else {
                panic!("expected an asserted subject");
            };
            assert_eq!(agent.entity.ty.as_str(), "FerrisKey::Agent");
            assert_eq!(agent.properties["client_id"], json!("claude"));
            assert_eq!(request.resource.entity.ty.as_str(), "FerrisKey::Prompt");
            assert_eq!(request.resource.properties["name"], json!("summarise"));
        }
    }

    #[test]
    fn a_ferriskey_subject_drops_its_properties() {
        let request = parse(json!({
            "subject": {"type": "FerrisKey::User", "id": "u", "properties": {"roles": ["admin"]}},
            "action": {"name": "document:read"},
            "resource": {"type": "App::Document", "id": "1", "properties": {"a": 1}},
        }))
        .expect("valid");
        assert_eq!(
            request.subject,
            Subject::FerrisKey(EntityRef::new(EntityType::custom("FerrisKey::User"), "u"))
        );
        assert_eq!(request.resource.properties.len(), 1);
        assert!(request.context.is_empty());
    }

    #[test]
    fn a_tenant_subject_keeps_its_properties() {
        let request = parse(json!({
            "subject": {"type": "App::Service", "id": "s", "properties": {"tier": "gold"}},
            "action": {"name": "x", "properties": {"ignored": true}},
            "resource": {"type": "App::Document", "id": "1"},
            "context": {"ip": "10.0.0.1"},
        }))
        .expect("valid");
        match request.subject {
            Subject::Tenant(entity) => assert_eq!(entity.properties.len(), 1),
            other => panic!("unexpected subject {other:?}"),
        }
        assert_eq!(request.context.len(), 1);
    }

    #[test]
    fn a_missing_type_is_rejected() {
        assert!(
            parse(json!({
                "subject": {"id": "u"},
                "action": {"name": "x"},
                "resource": {"type": "App::Document", "id": "1"},
            }))
            .is_err()
        );
    }

    #[test]
    fn an_empty_action_name_is_rejected() {
        assert!(
            parse(json!({
                "subject": {"type": "FerrisKey::User", "id": "u"},
                "action": {"name": ""},
                "resource": {"type": "App::Document", "id": "1"},
            }))
            .is_err()
        );
    }
}
