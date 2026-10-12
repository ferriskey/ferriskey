use std::collections::HashSet;
use std::str::FromStr;

use cedar_policy::{
    AuthorizationError, Authorizer, Context, Decision as CedarDecision, Entities, EntityId,
    EntityTypeName, EntityUid, PolicyId, PolicySet, Request, Schema,
};
use ferriskey_authz::entities::{Decision, EntityRef, EntityType};
use ferriskey_authz::tenant::evaluation::{
    ClientFacts, EvaluationRequest, GroupFacts, OrganizationFacts, PrincipalFacts, PrincipalSlice,
    RoleFacts, RuleSet, Subject, TenantEntity, UserFacts,
};
use ferriskey_authz::tenant::names::Namespace;
use ferriskey_authz::tenant::ports::TenantDecisionEngine;
use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::compiler::{CedarPolicyCompiler, build_policy_set, compile_schema};

#[derive(Debug)]
pub struct CompiledRules {
    schema: Option<Schema>,
    policies: PolicySet,
    namespace: String,
}

fn invalid(error: impl std::error::Error) -> CoreError {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    CoreError::InvalidAuthorizationRequest(message)
}

fn uid(entity: &EntityRef) -> Result<EntityUid, CoreError> {
    let ty = EntityTypeName::from_str(entity.ty.as_str()).map_err(invalid)?;
    Ok(EntityUid::from_type_name_and_id(
        ty,
        EntityId::new(&entity.id),
    ))
}

fn reference(entity: &EntityRef) -> Value {
    json!({ "type": entity.ty.as_str(), "id": entity.id })
}

fn entity_json(
    entity: &EntityRef,
    attrs: Map<String, Value>,
    parents: &[EntityRef],
    tags: Option<Map<String, Value>>,
) -> Value {
    let mut value = json!({
        "uid": reference(entity),
        "attrs": attrs,
        "parents": parents.iter().map(reference).collect::<Vec<_>>(),
    });
    if let (Some(tags), Some(object)) = (tags, value.as_object_mut()) {
        object.insert("tags".to_string(), Value::Object(tags));
    }
    value
}

fn attrs<const N: usize>(pairs: [(&str, Value); N]) -> Map<String, Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

fn fk(ty: &'static str, id: Uuid) -> EntityRef {
    EntityRef::new(EntityType::new(ty), id.to_string())
}

fn fk_user(id: Uuid) -> EntityRef {
    fk("FerrisKey::User", id)
}

fn fk_client(id: Uuid) -> EntityRef {
    fk("FerrisKey::Client", id)
}

fn fk_role(id: Uuid) -> EntityRef {
    fk("FerrisKey::Role", id)
}

fn fk_group(id: Uuid) -> EntityRef {
    fk("FerrisKey::Group", id)
}

fn fk_organization(id: Uuid) -> EntityRef {
    fk("FerrisKey::Organization", id)
}

fn refs(ids: &[Uuid], make: fn(Uuid) -> EntityRef) -> Vec<EntityRef> {
    ids.iter().copied().map(make).collect()
}

fn user_json(user: &UserFacts) -> Value {
    let mut map = attrs([
        ("username", json!(user.username)),
        ("email_verified", json!(user.email_verified)),
        ("enabled", json!(user.enabled)),
    ]);
    if let Some(email) = &user.email {
        map.insert("email".to_string(), json!(email));
    }
    let parents: Vec<EntityRef> = refs(&user.role_ids, fk_role)
        .into_iter()
        .chain(refs(&user.group_ids, fk_group))
        .chain(refs(&user.organization_ids, fk_organization))
        .collect();
    let tags: Map<String, Value> = user
        .tags
        .iter()
        .map(|(key, value)| (key.clone(), json!(value)))
        .collect();
    entity_json(&fk_user(user.id), map, &parents, Some(tags))
}

fn client_json(client: &ClientFacts) -> Value {
    entity_json(
        &fk_client(client.id),
        attrs([("client_id", json!(client.client_id))]),
        &refs(&client.role_ids, fk_role),
        None,
    )
}

fn role_json(role: &RoleFacts) -> Value {
    entity_json(
        &fk_role(role.id),
        attrs([("name", json!(role.name))]),
        &[],
        None,
    )
}

fn group_json(group: &GroupFacts) -> Value {
    let parents: Vec<EntityRef> = group
        .parent_id
        .map(fk_group)
        .into_iter()
        .chain([fk_organization(group.organization_id)])
        .collect();
    entity_json(
        &fk_group(group.id),
        attrs([("name", json!(group.name))]),
        &parents,
        None,
    )
}

fn organization_json(organization: &OrganizationFacts) -> Value {
    entity_json(
        &fk_organization(organization.id),
        attrs([
            ("alias", json!(organization.alias)),
            ("name", json!(organization.name)),
        ]),
        &[],
        None,
    )
}

fn tenant_json(entity: &TenantEntity) -> Value {
    entity_json(
        &entity.entity,
        entity.properties.clone(),
        &entity.parents,
        None,
    )
}

fn slice_json(slice: &PrincipalSlice) -> (EntityRef, Vec<Value>) {
    let (principal, subject) = match &slice.principal {
        PrincipalFacts::User(user) => (fk_user(user.id), user_json(user)),
        PrincipalFacts::Client(client) => (fk_client(client.id), client_json(client)),
    };
    let entities = std::iter::once(subject)
        .chain(slice.roles.iter().map(role_json))
        .chain(slice.groups.iter().map(group_json))
        .chain(slice.organizations.iter().map(organization_json))
        .collect();
    (principal, entities)
}

fn entity_key(value: &Value) -> String {
    value.get("uid").map(Value::to_string).unwrap_or_default()
}

fn dedupe(entities: Vec<Value>) -> Vec<Value> {
    let mut seen = HashSet::with_capacity(entities.len());
    entities
        .into_iter()
        .filter(|entity| seen.insert(entity_key(entity)))
        .collect()
}

fn policy_ids<'a>(ids: impl Iterator<Item = &'a PolicyId>) -> Vec<String> {
    let mut found: Vec<String> = ids.map(ToString::to_string).collect();
    found.sort();
    found.dedup();
    found
}

impl TenantDecisionEngine for CedarPolicyCompiler {
    type Compiled = CompiledRules;

    fn compile(&self, rules: &RuleSet) -> Result<Self::Compiled, Vec<PolicyIssue>> {
        let schema = rules.schema.as_ref().map(compile_schema).transpose()?;
        let (policies, _origins) = build_policy_set(rules.enabled_policies(), &rules.templates)?;
        let namespace = rules
            .schema
            .as_ref()
            .map_or_else(Namespace::default, |s| s.namespace.clone())
            .to_string();
        Ok(CompiledRules {
            schema,
            policies,
            namespace,
        })
    }

    fn decide(
        &self,
        compiled: &Self::Compiled,
        slice: Option<&PrincipalSlice>,
        request: &EvaluationRequest,
    ) -> Result<Decision, CoreError> {
        let schema = compiled.schema.as_ref();

        let (principal, mut entities) = match &request.subject {
            Subject::FerrisKey(_) => {
                let slice = slice.ok_or(CoreError::InternalServerError)?;
                slice_json(slice)
            }
            Subject::Tenant(subject) => (subject.entity.clone(), vec![tenant_json(subject)]),
        };
        entities.push(tenant_json(&request.resource));

        let entities =
            Entities::from_json_value(Value::Array(dedupe(entities)), schema).map_err(invalid)?;

        let action = EntityUid::from_type_name_and_id(
            EntityTypeName::from_str(&format!("{}::Action", compiled.namespace))
                .map_err(|_| CoreError::InternalServerError)?,
            EntityId::new(&request.action),
        );
        let context = Context::from_json_value(
            Value::Object(request.context.clone()),
            schema.map(|s| (s, &action)),
        )
        .map_err(invalid)?;
        let cedar_request = Request::new(
            uid(&principal)?,
            action,
            uid(&request.resource.entity)?,
            context,
            schema,
        )
        .map_err(invalid)?;

        let response =
            Authorizer::new().is_authorized(&cedar_request, &compiled.policies, &entities);
        let diagnostics = response.diagnostics();
        let erroring: Vec<&PolicyId> = diagnostics
            .errors()
            .map(|error| match error {
                AuthorizationError::PolicyEvaluationError(e) => e.policy_id(),
            })
            .collect();
        if !erroring.is_empty() {
            return Ok(Decision::deny().with_policies(policy_ids(erroring.into_iter())));
        }
        let determining = policy_ids(diagnostics.reason());
        Ok(Decision::from(response.decision() == CedarDecision::Allow).with_policies(determining))
    }
}
