use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use chrono::Utc;
use ferriskey_authz::entities::{Decision, EntityRef, EntityType, PolicyOrigin};
use ferriskey_authz::tenant::entities::{AuthzSchema, Policy, PolicyBody};
use ferriskey_authz::tenant::evaluation::{
    ClientFacts, EvaluationRequest, GroupFacts, OrganizationFacts, PrincipalFacts, PrincipalSlice,
    RoleFacts, RuleSet, Subject, TenantEntity, UserFacts,
};
use ferriskey_authz::tenant::names::{Namespace, PolicyName};
use ferriskey_authz::tenant::ports::TenantDecisionEngine;
use ferriskey_authz_cedar::CedarPolicyCompiler;
use ferriskey_domain::common::app_errors::CoreError;
use serde::Deserialize;
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Deserialize)]
struct Case {
    schema: Option<String>,
    policies: Vec<PolicyCase>,
    slice: Option<SliceCase>,
    request: RequestCase,
    expect: Expect,
}

#[derive(Deserialize, Clone)]
struct PolicyCase {
    name: String,
    source: String,
}

#[derive(Deserialize)]
struct SliceCase {
    user: Option<UserCase>,
    client: Option<ClientCase>,
    roles: Vec<RoleCase>,
    groups: Vec<GroupCase>,
    organizations: Vec<OrganizationCase>,
}

#[derive(Deserialize)]
struct UserCase {
    id: Uuid,
    username: String,
    email: Option<String>,
    email_verified: bool,
    enabled: bool,
    tags: BTreeMap<String, String>,
    roles: Vec<Uuid>,
    groups: Vec<Uuid>,
    organizations: Vec<Uuid>,
}

#[derive(Deserialize)]
struct ClientCase {
    id: Uuid,
    client_id: String,
    roles: Vec<Uuid>,
}

#[derive(Deserialize)]
struct RoleCase {
    id: Uuid,
    name: String,
}

#[derive(Deserialize)]
struct GroupCase {
    id: Uuid,
    name: String,
    parent: Option<Uuid>,
    organization: Uuid,
}

#[derive(Deserialize)]
struct OrganizationCase {
    id: Uuid,
    alias: String,
    name: String,
}

#[derive(Deserialize)]
struct EntityCase {
    #[serde(rename = "type")]
    ty: String,
    id: String,
}

#[derive(Deserialize)]
struct TenantCase {
    entity: EntityCase,
    properties: Map<String, Value>,
    parents: Vec<EntityCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubjectCase {
    Ferriskey,
    Tenant(TenantCase),
}

#[derive(Deserialize)]
struct RequestCase {
    subject: SubjectCase,
    action: String,
    resource: TenantCase,
    context: Map<String, Value>,
}

#[derive(Deserialize)]
struct Expect {
    outcome: Outcome,
    policies: Vec<String>,
    message: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Allow,
    Deny,
    InvalidRequest,
    InternalError,
}

fn entity_ref(case: &EntityCase) -> EntityRef {
    EntityRef::new(EntityType::custom(case.ty.clone()), case.id.clone())
}

fn tenant(case: &TenantCase) -> TenantEntity {
    TenantEntity {
        entity: entity_ref(&case.entity),
        properties: case.properties.clone(),
        parents: case.parents.iter().map(entity_ref).collect(),
    }
}

fn principal(slice: &SliceCase) -> PrincipalFacts {
    match (&slice.user, &slice.client) {
        (Some(user), _) => PrincipalFacts::User(UserFacts {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            email_verified: user.email_verified,
            enabled: user.enabled,
            tags: user.tags.clone(),
            role_ids: user.roles.clone(),
            group_ids: user.groups.clone(),
            organization_ids: user.organizations.clone(),
        }),
        (None, Some(client)) => PrincipalFacts::Client(ClientFacts {
            id: client.id,
            client_id: client.client_id.clone(),
            role_ids: client.roles.clone(),
        }),
        (None, None) => panic!("a slice needs a user or a client"),
    }
}

fn slice(case: &SliceCase) -> PrincipalSlice {
    PrincipalSlice {
        principal: principal(case),
        roles: case
            .roles
            .iter()
            .map(|r| RoleFacts {
                id: r.id,
                name: r.name.clone(),
            })
            .collect(),
        groups: case
            .groups
            .iter()
            .map(|g| GroupFacts {
                id: g.id,
                name: g.name.clone(),
                parent_id: g.parent,
                organization_id: g.organization,
            })
            .collect(),
        organizations: case
            .organizations
            .iter()
            .map(|o| OrganizationFacts {
                id: o.id,
                alias: o.alias.clone(),
                name: o.name.clone(),
            })
            .collect(),
    }
}

fn subject(case: &Case, slice: Option<&PrincipalSlice>) -> Subject {
    match (&case.request.subject, slice) {
        (SubjectCase::Tenant(entity), _) => Subject::Tenant(tenant(entity)),
        (SubjectCase::Ferriskey, Some(slice)) => Subject::FerrisKey(match &slice.principal {
            PrincipalFacts::User(u) => EntityRef::user(u.id),
            PrincipalFacts::Client(c) => EntityRef::client(c.id),
        }),
        (SubjectCase::Ferriskey, None) => Subject::FerrisKey(EntityRef::user(Uuid::nil())),
    }
}

fn policy(case: &PolicyCase) -> Policy {
    Policy {
        id: Uuid::new_v4(),
        name: PolicyName::try_from(case.name.clone()).expect("policy name"),
        description: None,
        origin: PolicyOrigin::Custom,
        enabled: true,
        body: PolicyBody::Static {
            source: case.source.clone(),
        },
    }
}

fn rules(schema: Option<&str>, policies: Vec<Policy>) -> RuleSet {
    RuleSet {
        version: 1,
        schema: schema.map(|source| AuthzSchema {
            id: Uuid::new_v4(),
            namespace: Namespace::default(),
            source: source.to_string(),
            updated_at: Utc::now(),
        }),
        policies,
        templates: Vec::new(),
    }
}

fn run(
    schema: Option<&str>,
    policies: Vec<Policy>,
    slice: Option<&PrincipalSlice>,
    case: &Case,
) -> (Result<Decision, CoreError>, BTreeMap<String, String>) {
    let names: BTreeMap<String, String> = policies
        .iter()
        .map(|p| (p.id.to_string(), p.name.to_string()))
        .collect();
    let engine = CedarPolicyCompiler::new();
    let compiled = engine
        .compile(&rules(schema, policies))
        .expect("the rules compile");
    let request = EvaluationRequest {
        subject: subject(case, slice),
        action: case.request.action.clone(),
        resource: tenant(&case.request.resource),
        context: case.request.context.clone(),
    };
    (engine.decide(&compiled, slice, &request), names)
}

fn named(decision: &Decision, names: &BTreeMap<String, String>) -> BTreeSet<String> {
    decision
        .determining_policies
        .iter()
        .map(|id| names.get(id).cloned().unwrap_or_else(|| id.clone()))
        .collect()
}

fn check(file: &str, case: &Case) -> Result<(), String> {
    let slice = case.slice.as_ref().map(slice);
    let policies: Vec<Policy> = case.policies.iter().map(policy).collect();
    let (result, names) = run(case.schema.as_deref(), policies, slice.as_ref(), case);
    let expected: BTreeSet<String> = case.expect.policies.iter().cloned().collect();

    match (&case.expect.outcome, &result) {
        (Outcome::Allow, Ok(d)) if d.allowed && named(d, &names) == expected => Ok(()),
        (Outcome::Deny, Ok(d)) if !d.allowed && named(d, &names) == expected => Ok(()),
        (Outcome::InvalidRequest, Err(CoreError::InvalidAuthorizationRequest(message)))
            if case
                .expect
                .message
                .as_deref()
                .is_none_or(|needle| message.contains(needle)) =>
        {
            Ok(())
        }
        (Outcome::InternalError, Err(CoreError::InternalServerError)) => Ok(()),
        _ => Err(format!(
            "{file}: expected {:?} with {expected:?}, got {:?}",
            case.expect.outcome,
            result.map(|d| (d.allowed, named(&d, &names)))
        )),
    }
}

fn load(path: &Path) -> Case {
    let text = fs::read_to_string(path).expect("case file");
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn every_case_in_the_corpus_holds() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("cases directory")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    assert!(files.len() >= 20, "the corpus shrank to {}", files.len());

    let failures: Vec<String> = files
        .iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().into_owned();
            check(&name, &load(path)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    (0..items.len())
        .flat_map(|i| {
            let mut rest = items.to_vec();
            let head = rest.remove(i);
            permutations(&rest).into_iter().map(move |mut tail| {
                tail.insert(0, head.clone());
                tail
            })
        })
        .collect()
}

#[test]
fn the_order_of_policies_never_changes_the_decision() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/abac-tag-allow.json");
    let mut case = load(&path);
    case.policies = vec![
        PolicyCase {
            name: "permit-legal".into(),
            source: r#"permit (principal, action, resource) when { principal.hasTag("department") && principal.getTag("department") == "legal" };"#.into(),
        },
        PolicyCase {
            name: "permit-any-user".into(),
            source: "permit (principal is FerrisKey::User, action, resource);".into(),
        },
        PolicyCase {
            name: "forbid-sales".into(),
            source: r#"forbid (principal, action, resource) when { principal.hasTag("department") && principal.getTag("department") == "sales" };"#.into(),
        },
        PolicyCase {
            name: "forbid-disabled".into(),
            source: "forbid (principal, action, resource) when { !principal.enabled };".into(),
        },
    ];
    let slices = ["legal", "sales"].map(|department| {
        let mut slice = load(&path).slice.expect("slice");
        if let Some(user) = slice.user.as_mut() {
            user.tags = BTreeMap::from([("department".to_string(), department.to_string())]);
        }
        slice
    });

    for slice_case in &slices {
        let facts = slice(slice_case);
        let mut outcomes = BTreeSet::new();
        for order in permutations(&case.policies) {
            let policies: Vec<Policy> = order.iter().map(policy).collect();
            let (result, names) = run(case.schema.as_deref(), policies, Some(&facts), &case);
            let decision = result.expect("a decision");
            outcomes.insert((decision.allowed, named(&decision, &names)));
        }
        assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    }
}
