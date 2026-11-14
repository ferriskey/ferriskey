use chrono::Utc;
use ferriskey_authz::entities::{EntityRef, EntityType, PolicyOrigin};
use ferriskey_authz::tenant::entities::{AuthzSchema, Policy, PolicyBody};
use ferriskey_authz::tenant::evaluation::{EvaluationRequest, RuleSet, Subject, TenantEntity};
use ferriskey_authz::tenant::names::{Namespace, PolicyName};
use ferriskey_authz::tenant::ports::{PolicyCompiler, TenantDecisionEngine};
use ferriskey_authz_cedar::CedarPolicyCompiler;
use serde_json::{Map, Value, json};
use uuid::Uuid;

const REALM_SCHEMA: &str = r#"namespace App {
  type UserExtension = {
    on_call: Bool,
    on_call_schedules: Set<String>,
  };
  type ToolExtension = {
    access: String,
    destructive: Bool,
    organisation?: String,
  };
  type ContextExtension = {
    target: { service?: String },
    time: {
      iso?: String,
      minute: Long,
      timezone: String,
      iso_utc: String,
    },
  };
}"#;

const POLICY: &str = r#"permit (
  principal is FerrisKey::User,
  action == FerrisKey::Action::"call_tool",
  resource is FerrisKey::Tool
) when {
  principal.on_call &&
  principal.on_call_schedules.contains("primary") &&
  resource.access == "write" &&
  !resource.destructive &&
  context.target has service &&
  context.target.service == "billing" &&
  context.time.minute < 30 &&
  context.time.timezone == "Europe/Paris" &&
  context.time.iso_utc like "2026*"
};"#;

fn policy(name: &str, source: &str) -> Policy {
    Policy {
        id: Uuid::new_v4(),
        name: PolicyName::try_from(name.to_string()).expect("policy name"),
        description: None,
        origin: PolicyOrigin::Custom,
        enabled: true,
        managed_by: None,
        body: PolicyBody::Static {
            source: source.to_string(),
        },
    }
}

fn schema(source: &str) -> AuthzSchema {
    AuthzSchema {
        id: Uuid::new_v4(),
        namespace: Namespace::default(),
        source: source.to_string(),
        updated_at: Utc::now(),
    }
}

fn object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        other => panic!("not an object: {other}"),
    }
}

fn entity(ty: &str, id: &str, properties: Value) -> TenantEntity {
    TenantEntity {
        entity: EntityRef::new(EntityType::custom(ty.to_string()), id),
        properties: object(properties),
        parents: Vec::new(),
    }
}

fn user(properties: Value) -> TenantEntity {
    let mut base = object(json!({
        "username": "u-1",
        "email_verified": false,
        "enabled": true,
        "roles": [],
    }));
    base.extend(object(properties));
    entity("FerrisKey::User", "u-1", Value::Object(base))
}

fn tool(properties: Value) -> TenantEntity {
    let mut base = object(json!({"server": "billing", "tool": "refund"}));
    base.extend(object(properties));
    entity("FerrisKey::Tool", "billing/refund", Value::Object(base))
}

fn request(subject: TenantEntity, resource: TenantEntity, context: Value) -> EvaluationRequest {
    EvaluationRequest {
        subject: Subject::Asserted(subject),
        action: "call_tool".to_string(),
        resource,
        context: object(context),
    }
}

fn decide(
    schema_source: Option<&str>,
    policies: Vec<Policy>,
    request: &EvaluationRequest,
) -> Result<bool, String> {
    let engine = CedarPolicyCompiler::new();
    let compiled = engine
        .compile(&RuleSet {
            version: 1,
            schema: schema_source.map(schema),
            policies,
            templates: Vec::new(),
        })
        .expect("the rules compile");
    engine
        .decide(&compiled, None, request)
        .map(|d| d.allowed)
        .map_err(|e| e.to_string())
}

fn gateway_context() -> Value {
    json!({
        "agent": "gateway",
        "delegated": true,
        "arguments": {"invoice": "inv-7", "amount": 12, "dry_run": false},
        "target": {"service": "billing", "region": "eu"},
        "time": {
            "weekday": 3,
            "hour": 10,
            "minute": 5,
            "timezone": "Europe/Paris",
            "iso_utc": "2026-11-03T09:05:00Z",
        },
    })
}

fn full_request() -> EvaluationRequest {
    request(
        user(json!({"on_call": true, "on_call_schedules": ["primary"], "nickname": "x"})),
        tool(json!({"access": "write", "destructive": false, "organisation": "acme"})),
        gateway_context(),
    )
}

#[test]
fn without_a_schema_every_property_and_context_key_reaches_the_policy() {
    let policies = vec![policy("on-call-writes", POLICY)];
    assert_eq!(decide(None, policies.clone(), &full_request()), Ok(true));

    let off_call = request(
        user(json!({"on_call": false, "on_call_schedules": ["primary"]})),
        tool(json!({"access": "write", "destructive": false})),
        gateway_context(),
    );
    assert_eq!(decide(None, policies, &off_call), Ok(false));
}

#[test]
fn without_a_schema_nulls_and_floats_are_left_out_instead_of_failing() {
    let policies = vec![policy(
        "reads",
        r#"permit (principal, action == FerrisKey::Action::"call_tool", resource) when { context.arguments.invoice == "inv-7" };"#,
    )];
    let noisy = request(
        user(json!({"on_call": null, "ratio": 0.5})),
        tool(json!({"access": null})),
        json!({"arguments": {"invoice": "inv-7", "score": 1.5, "gone": null}, "extra": null}),
    );
    assert_eq!(decide(None, policies, &noisy), Ok(true));
}

#[test]
fn a_policy_reading_an_absent_attribute_denies_instead_of_failing() {
    let policies = vec![policy("on-call-writes", POLICY)];
    let bare = request(user(json!({})), tool(json!({})), gateway_context());
    assert_eq!(decide(None, policies, &bare), Ok(false));
}

#[test]
fn a_realm_schema_declares_the_attributes_its_policies_use() {
    let compiler = CedarPolicyCompiler::new();
    let policies = vec![policy("on-call-writes", POLICY)];
    assert_eq!(
        compiler.validate_policies(Some(&schema(REALM_SCHEMA)), &policies, &[]),
        Ok(())
    );
    assert_eq!(
        decide(Some(REALM_SCHEMA), policies.clone(), &full_request()),
        Ok(true)
    );

    let late = request(
        user(json!({"on_call": true, "on_call_schedules": ["primary"]})),
        tool(json!({"access": "write", "destructive": false})),
        {
            let mut context = gateway_context();
            context["time"]["minute"] = json!(45);
            context
        },
    );
    assert_eq!(decide(Some(REALM_SCHEMA), policies, &late), Ok(false));
}

#[test]
fn a_policy_using_an_undeclared_attribute_is_rejected_when_a_schema_is_stored() {
    let compiler = CedarPolicyCompiler::new();
    let policies = vec![policy(
        "undeclared",
        r#"permit (principal is FerrisKey::User, action, resource) when { principal.pager == "x" };"#,
    )];
    let issues = compiler
        .validate_policies(Some(&schema(REALM_SCHEMA)), &policies, &[])
        .expect_err("undeclared attribute");
    assert_eq!(issues[0].policy.as_deref(), Some("undeclared"));
}

#[test]
fn undeclared_properties_and_context_keys_are_ignored_when_a_schema_is_stored() {
    let policies = vec![policy(
        "any-call",
        r#"permit (principal, action == FerrisKey::Action::"call_tool", resource);"#,
    )];
    let mut call = full_request();
    call.context["time"]["iso"] = json!("2026-11-03T09:05:00Z");
    assert_eq!(decide(Some("namespace App {}"), policies, &call), Ok(true));
}

#[test]
fn a_declared_attribute_of_the_wrong_type_is_a_bad_request() {
    let policies = vec![policy("on-call-writes", POLICY)];
    let wrong = request(
        user(json!({"on_call": "yes"})),
        tool(json!({})),
        gateway_context(),
    );
    assert!(decide(Some(REALM_SCHEMA), policies, &wrong).is_err());
}

#[test]
fn an_extension_cannot_change_the_type_of_a_built_in_attribute() {
    let compiler = CedarPolicyCompiler::new();
    let issues = compiler
        .validate_schema(&schema(
            "namespace App { type UserExtension = { username: Long }; }",
        ))
        .expect_err("conflicting type");
    assert!(issues[0].message.contains("built-in"), "{issues:?}");
}

#[test]
fn an_extension_may_relax_a_required_built_in_context_key() {
    let compiler = CedarPolicyCompiler::new();
    let schema_source = "namespace App { type ContextExtension = { time: { iso?: String } }; }";
    assert_eq!(compiler.validate_schema(&schema(schema_source)), Ok(()));
    let policies = vec![policy(
        "guarded",
        r#"permit (principal, action == FerrisKey::Action::"call_tool", resource) when { context.time has iso && context.time.iso == "now" };"#,
    )];
    assert_eq!(
        compiler.validate_policies(Some(&schema(schema_source)), &policies, &[]),
        Ok(())
    );
}

#[test]
fn a_realm_without_extensions_decides_as_before() {
    let policies = vec![policy(
        "support-read-only",
        r#"permit (principal in FerrisKey::Group::"support", action == FerrisKey::Action::"call_tool", resource is FerrisKey::Tool)
when { context.agent == "claude" && context.time.hour >= 8 && resource has annotations && resource.annotations has readOnlyHint && resource.annotations.readOnlyHint };"#,
    )];
    let mut member = user(json!({}));
    member.parents = vec![EntityRef::new(
        EntityType::custom("FerrisKey::Group".to_string()),
        "support",
    )];
    let call = request(
        member,
        tool(json!({"annotations": {"readOnlyHint": true}})),
        json!({
            "agent": "claude",
            "delegated": true,
            "arguments": {"path": "/tmp/report.txt"},
            "time": {"weekday": 3, "hour": 10, "iso": "2026-10-22T10:00:00Z"},
        }),
    );
    assert_eq!(
        decide(Some("namespace App {}"), policies.clone(), &call),
        Ok(true)
    );
    assert_eq!(decide(None, policies, &call), Ok(true));
}
