use ferriskey_authz::entities::{EntityRef, EntityType};
use ferriskey_authz::tenant::FERRISKEY_NAMESPACE;
use ferriskey_authz::tenant::evaluation::{EvaluationRequest, Subject, TenantEntity};
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

impl TryFrom<AuthzenEntity> for TenantEntity {
    type Error = CoreError;

    fn try_from(value: AuthzenEntity) -> Result<Self, Self::Error> {
        required("type", &value.entity_type)?;
        required("id", &value.id)?;
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
