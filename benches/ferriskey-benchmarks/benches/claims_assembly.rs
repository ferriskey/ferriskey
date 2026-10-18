use std::collections::HashMap;

use chrono::Utc;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ferriskey_aegis::entities::{ClientScope, ClientScopeSet, ProtocolMapper, ScopeType};
use ferriskey_core::domain::authentication::mapper_engine::{
    MapperContext, MapperEngine, TokenType,
};
use ferriskey_core::domain::authentication::scope::resolve_requested_scope;
use ferriskey_domain::realm::RealmId;
use serde_json::{Value, json};
use uuid::Uuid;

fn mapper(scope_id: Uuid, name: &str, mapper_type: &str, config: Value) -> ProtocolMapper {
    ProtocolMapper {
        id: Uuid::new_v4(),
        client_scope_id: scope_id,
        name: name.to_string(),
        mapper_type: mapper_type.to_string(),
        config,
        created_at: Utc::now(),
    }
}

fn property(scope_id: Uuid, claim: &str, attribute: &str) -> ProtocolMapper {
    mapper(
        scope_id,
        claim,
        "oidc-usermodel-property-mapper",
        json!({
            "user.attribute": attribute,
            "claim.name": claim,
            "access.token.claim": "true",
            "id.token.claim": "true"
        }),
    )
}

fn default_scope_mappers() -> Vec<ProtocolMapper> {
    let profile = Uuid::new_v4();
    let email = Uuid::new_v4();
    let roles = Uuid::new_v4();
    vec![
        property(profile, "given_name", "firstName"),
        property(profile, "family_name", "lastName"),
        property(profile, "preferred_username", "username"),
        property(email, "email", "email"),
        mapper(
            email,
            "email_verified",
            "oidc-usermodel-property-mapper",
            json!({
                "user.attribute": "emailVerified",
                "claim.name": "email_verified",
                "jsonType.label": "boolean",
                "access.token.claim": "true",
                "id.token.claim": "true"
            }),
        ),
        mapper(
            roles,
            "realm_access",
            "oidc-usermodel-realm-role-mapper",
            json!({
                "claim.name": "realm_access.roles",
                "access.token.claim": "true",
                "id.token.claim": "true"
            }),
        ),
    ]
}

fn context(realm_id: RealmId) -> MapperContext {
    MapperContext {
        user_id: Uuid::new_v4(),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        email_verified: true,
        firstname: "Alice".to_string(),
        lastname: "Martin".to_string(),
        realm_roles: vec!["default-roles".to_string(), "offline_access".to_string()],
        client_roles: HashMap::from([(
            "security-admin-console".to_string(),
            vec!["manage-users".to_string(), "view-realm".to_string()],
        )]),
        client_id: "security-admin-console".to_string(),
        client_uuid: Uuid::new_v4(),
        realm_name: "master".to_string(),
        realm_id,
        user_attributes: HashMap::new(),
        organizations: Vec::new(),
        groups: Vec::new(),
    }
}

fn scope(realm_id: RealmId, name: &str, scope_type: ScopeType) -> ClientScope {
    ClientScope {
        id: Uuid::new_v4(),
        realm_id,
        name: name.to_string(),
        description: None,
        protocol: "openid-connect".to_string(),
        default_scope_type: scope_type,
        dynamic_registration_allowed: false,
        attributes: None,
        protocol_mappers: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn scope_set(realm_id: RealmId) -> ClientScopeSet {
    ClientScopeSet {
        default: ["openid", "profile", "email", "roles"]
            .into_iter()
            .map(|name| scope(realm_id, name, ScopeType::Default))
            .collect(),
        optional: ["address", "phone", "offline_access", "organization"]
            .into_iter()
            .map(|name| scope(realm_id, name, ScopeType::Optional))
            .collect(),
    }
}

fn claims_assembly(c: &mut Criterion) {
    let realm_id = RealmId::new(Uuid::new_v4());
    let engine = MapperEngine::new();
    let mappers = default_scope_mappers();
    let context = context(realm_id);
    let set = scope_set(realm_id);

    c.bench_function("apply the default scope mappers to an access token", |b| {
        b.iter(|| {
            engine
                .apply_mappers(black_box(&mappers), &context, TokenType::AccessToken)
                .expect("apply the mappers")
        })
    });

    c.bench_function(
        "resolve a requested scope against a client's scope set",
        |b| {
            b.iter(|| {
                resolve_requested_scope(black_box(&set), Some("openid profile email phone"))
                    .expect("resolve the scope")
            })
        },
    );
}

criterion_group!(benches, claims_assembly);
criterion_main!(benches);
