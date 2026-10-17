use std::collections::BTreeMap;
use std::hint::black_box;

use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use ferriskey_authz::entities::{EntityRef, EntityType, PolicyOrigin};
use ferriskey_authz::tenant::entities::{AuthzSchema, Policy, PolicyBody};
use ferriskey_authz::tenant::evaluation::{
    EvaluationRequest, GroupFacts, OrganizationFacts, PrincipalFacts, PrincipalSlice, RoleFacts,
    RuleSet, Subject, TenantEntity, UserFacts,
};
use ferriskey_authz::tenant::names::{Namespace, PolicyName};
use ferriskey_authz::tenant::ports::TenantDecisionEngine;
use ferriskey_authz_cedar::CedarPolicyCompiler;
use serde_json::{Map, json};
use uuid::Uuid;

const ACTIONS: usize = 10;
const RBAC_POLICIES: usize = 900;
const ABAC_POLICIES: usize = 90;
const FORBID_POLICIES: usize = 10;
const HELD_ROLES: usize = 500;
const ORGANIZATIONS: usize = 40;
const GROUP_DEPTH: usize = 10;
const EXTRA_ORGANIZATIONS: usize = 59;

fn schema() -> AuthzSchema {
    let actions: Vec<String> = (0..ACTIONS)
        .map(|i| {
            format!(
                "  action \"a{i}\" appliesTo {{ principal: [FerrisKey::User], resource: [Document] }};"
            )
        })
        .collect();
    AuthzSchema {
        id: Uuid::new_v4(),
        namespace: Namespace::default(),
        source: format!(
            "namespace App {{\n  entity Document = {{ owner: FerrisKey::User, classification: Long }};\n{}\n}}\n",
            actions.join("\n")
        ),
        updated_at: Utc::now(),
    }
}

fn policy(index: usize, source: String) -> Policy {
    Policy {
        id: Uuid::new_v4(),
        name: PolicyName::try_from(format!("p{index}")).unwrap_or_else(|_| unreachable!()),
        description: None,
        origin: PolicyOrigin::Custom,
        enabled: true,
        body: PolicyBody::Static { source },
    }
}

fn rules(role_ids: &[Uuid]) -> RuleSet {
    let rbac = role_ids.iter().enumerate().map(|(i, role)| {
        format!(
            "permit (principal in FerrisKey::Role::\"{role}\", action == App::Action::\"a{}\", resource is App::Document);",
            i % ACTIONS
        )
    });
    let abac = (0..ABAC_POLICIES).map(|i| {
        format!(
            "permit (principal, action == App::Action::\"a{}\", resource) when {{ resource.owner == principal && resource.classification < {i} }};",
            i % ACTIONS
        )
    });
    let forbid = (0..FORBID_POLICIES).map(|i| {
        format!("forbid (principal, action, resource) when {{ principal.hasTag(\"blocked{i}\") }};")
    });
    RuleSet {
        version: 1,
        schema: Some(schema()),
        policies: rbac
            .chain(abac)
            .chain(forbid)
            .enumerate()
            .map(|(i, source)| policy(i, source))
            .collect(),
        templates: Vec::new(),
    }
}

fn slice(user_id: Uuid, role_ids: &[Uuid]) -> PrincipalSlice {
    let held = &role_ids[RBAC_POLICIES - HELD_ROLES..];
    let organizations: Vec<OrganizationFacts> = (0..ORGANIZATIONS + EXTRA_ORGANIZATIONS)
        .map(|i| OrganizationFacts {
            id: Uuid::new_v4(),
            alias: format!("org{i}"),
            name: format!("Org {i}"),
        })
        .collect();
    let mut groups = Vec::with_capacity(ORGANIZATIONS * GROUP_DEPTH);
    let mut leaves = Vec::with_capacity(ORGANIZATIONS);
    for organization in &organizations[..ORGANIZATIONS] {
        let mut parent = None;
        for depth in 0..GROUP_DEPTH {
            let id = Uuid::new_v4();
            groups.push(GroupFacts {
                id,
                name: format!("g{depth}"),
                parent_id: parent,
                organization_id: organization.id,
            });
            parent = Some(id);
        }
        leaves.extend(parent);
    }
    PrincipalSlice {
        principal: PrincipalFacts::User(UserFacts {
            id: user_id,
            username: "bench".to_string(),
            email: Some("bench@ferriskey.rs".to_string()),
            email_verified: true,
            enabled: true,
            tags: BTreeMap::from([("department".to_string(), "legal".to_string())]),
            role_ids: held.to_vec(),
            group_ids: leaves,
            organization_ids: organizations.iter().map(|o| o.id).collect(),
        }),
        roles: held
            .iter()
            .enumerate()
            .map(|(i, id)| RoleFacts {
                id: *id,
                name: format!("role{i}"),
            })
            .collect(),
        groups,
        organizations,
    }
}

fn request(user_id: Uuid, action: &str) -> EvaluationRequest {
    let mut properties = Map::new();
    properties.insert(
        "owner".to_string(),
        json!({ "__entity": { "type": "FerrisKey::User", "id": user_id.to_string() } }),
    );
    properties.insert("classification".to_string(), json!(50));
    EvaluationRequest {
        subject: Subject::FerrisKey(EntityRef::new(
            EntityType::new("FerrisKey::User"),
            user_id.to_string(),
        )),
        action: action.to_string(),
        resource: TenantEntity {
            entity: EntityRef::new(EntityType::new("App::Document"), "doc-1"),
            properties,
            parents: Vec::new(),
        },
        context: Map::new(),
    }
}

fn bench(c: &mut Criterion) {
    let engine = CedarPolicyCompiler::new();
    let role_ids: Vec<Uuid> = (0..RBAC_POLICIES).map(|_| Uuid::new_v4()).collect();
    let user_id = Uuid::new_v4();
    let rules = rules(&role_ids);
    let slice = slice(user_id, &role_ids);
    let compiled = engine
        .compile(&rules)
        .unwrap_or_else(|issues| panic!("{issues:?}"));
    let allowed = request(user_id, "a9");

    let decision = engine
        .decide(&compiled, Some(&slice), &allowed)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(decision.allowed);

    c.bench_function("decide 1000 policies x 1000 entities", |b| {
        b.iter(|| engine.decide(black_box(&compiled), Some(&slice), black_box(&allowed)))
    });
    c.bench_function("compile 1000 policies", |b| {
        b.iter(|| engine.compile(black_box(&rules)))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
