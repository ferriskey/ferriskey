use std::collections::HashSet;
use std::str::FromStr;

use cedar_policy::{
    AuthorizationError, Authorizer, Context, Decision as CedarDecision, Entities, Entity, EntityId,
    EntityTypeName, EntityUid, PolicyId, PolicySet, Request, RestrictedExpression, Schema,
};
use ferriskey_authz::entities::{Decision, EntityRef};
use ferriskey_authz::tenant::evaluation::{
    ClientFacts, EvaluationRequest, GroupFacts, OrganizationFacts, PrincipalFacts, PrincipalSlice,
    RoleFacts, RuleSet, Subject, TenantEntity, UserFacts,
};
use ferriskey_authz::tenant::names::Namespace;
use ferriskey_authz::tenant::ports::TenantDecisionEngine;
use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::compiler::{CedarPolicyCompiler, build_policy_set, compile_schema};

const USER: &str = "FerrisKey::User";
const CLIENT: &str = "FerrisKey::Client";
const ROLE: &str = "FerrisKey::Role";
const GROUP: &str = "FerrisKey::Group";
const ORGANIZATION: &str = "FerrisKey::Organization";

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

fn fk(ty: &'static str, id: Uuid) -> Result<EntityUid, CoreError> {
    EntityTypeName::from_str(ty)
        .map(|ty| EntityUid::from_type_name_and_id(ty, EntityId::new(id.to_string())))
        .map_err(|_| CoreError::InternalServerError)
}

fn fk_all(ty: &'static str, ids: &[Uuid]) -> Result<Vec<EntityUid>, CoreError> {
    ids.iter().map(|id| fk(ty, *id)).collect()
}

fn text(value: &str) -> RestrictedExpression {
    RestrictedExpression::new_string(value.to_string())
}

fn entity(
    uid: EntityUid,
    attrs: impl IntoIterator<Item = (&'static str, RestrictedExpression)>,
    parents: Vec<EntityUid>,
    tags: impl IntoIterator<Item = (String, RestrictedExpression)>,
) -> Result<Entity, CoreError> {
    Entity::new_with_tags(
        uid,
        attrs
            .into_iter()
            .map(|(key, value)| (key.to_string(), value)),
        parents,
        tags,
    )
    .map_err(|_| CoreError::InternalServerError)
}

fn user_entity(user: &UserFacts) -> Result<Entity, CoreError> {
    let mut attrs = vec![
        ("username", text(&user.username)),
        (
            "email_verified",
            RestrictedExpression::new_bool(user.email_verified),
        ),
        ("enabled", RestrictedExpression::new_bool(user.enabled)),
    ];
    if let Some(email) = &user.email {
        attrs.push(("email", text(email)));
    }
    let mut parents = fk_all(ROLE, &user.role_ids)?;
    parents.extend(fk_all(GROUP, &user.group_ids)?);
    parents.extend(fk_all(ORGANIZATION, &user.organization_ids)?);
    let tags = user
        .tags
        .iter()
        .map(|(key, value)| (key.clone(), text(value)));
    entity(fk(USER, user.id)?, attrs, parents, tags)
}

fn client_entity(client: &ClientFacts) -> Result<Entity, CoreError> {
    entity(
        fk(CLIENT, client.id)?,
        [("client_id", text(&client.client_id))],
        fk_all(ROLE, &client.role_ids)?,
        [],
    )
}

fn role_entity(role: &RoleFacts) -> Result<Entity, CoreError> {
    entity(
        fk(ROLE, role.id)?,
        [("name", text(&role.name))],
        Vec::new(),
        [],
    )
}

fn group_entity(group: &GroupFacts) -> Result<Entity, CoreError> {
    let mut parents = vec![fk(ORGANIZATION, group.organization_id)?];
    if let Some(parent) = group.parent_id {
        parents.push(fk(GROUP, parent)?);
    }
    entity(
        fk(GROUP, group.id)?,
        [("name", text(&group.name))],
        parents,
        [],
    )
}

fn organization_entity(organization: &OrganizationFacts) -> Result<Entity, CoreError> {
    entity(
        fk(ORGANIZATION, organization.id)?,
        [
            ("alias", text(&organization.alias)),
            ("name", text(&organization.name)),
        ],
        Vec::new(),
        [],
    )
}

fn tenant_entity(entity: &TenantEntity, schema: Option<&Schema>) -> Result<Entity, CoreError> {
    let value = json!({
        "uid": reference(&entity.entity),
        "attrs": entity.properties,
        "parents": entity.parents.iter().map(reference).collect::<Vec<_>>(),
    });
    Entity::from_json_value(value, schema).map_err(invalid)
}

fn slice_entities(slice: &PrincipalSlice) -> Result<(EntityUid, Vec<Entity>), CoreError> {
    let (principal, subject) = match &slice.principal {
        PrincipalFacts::User(user) => (fk(USER, user.id)?, user_entity(user)?),
        PrincipalFacts::Client(client) => (fk(CLIENT, client.id)?, client_entity(client)?),
    };
    let mut entities =
        Vec::with_capacity(1 + slice.roles.len() + slice.groups.len() + slice.organizations.len());
    entities.push(subject);
    for role in &slice.roles {
        entities.push(role_entity(role)?);
    }
    for group in &slice.groups {
        entities.push(group_entity(group)?);
    }
    for organization in &slice.organizations {
        entities.push(organization_entity(organization)?);
    }
    Ok((principal, entities))
}

fn dedupe(entities: Vec<Entity>) -> Vec<Entity> {
    let mut seen = HashSet::with_capacity(entities.len());
    entities
        .into_iter()
        .filter(|entity| seen.insert(entity.uid()))
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
            Subject::FerrisKey(_) => slice_entities(slice.ok_or(CoreError::InternalServerError)?)?,
            Subject::Tenant(subject) => {
                (uid(&subject.entity)?, vec![tenant_entity(subject, schema)?])
            }
        };
        entities.push(tenant_entity(&request.resource, schema)?);

        let entities = Entities::from_entities(dedupe(entities), schema).map_err(invalid)?;

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
            principal,
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
