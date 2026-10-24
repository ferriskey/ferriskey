use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct Rule {
    #[serde(rename = "apiGroups")]
    api_groups: Vec<String>,
    resources: Vec<String>,
    verbs: Vec<String>,
}

fn strip_template(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        if line.trim_start().starts_with("{{") {
            continue;
        }
        let mut rest = line;
        while let Some(start) = rest.find("{{") {
            out.push_str(&rest[..start]);
            out.push_str("placeholder");
            match rest[start..].find("}}") {
                Some(end) => rest = &rest[start + end + 2..],
                None => rest = "",
            }
        }
        out.push_str(rest);
        out.push('\n');
    }
    out
}

fn chart_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../charts/ferriskey-operator/templates")
        .join(relative)
}

fn read_template(relative: &str) -> String {
    std::fs::read_to_string(chart_path(relative)).expect("read the chart template")
}

const RULES_HELPER: &str = "ferriskey-operator.operator.rbacRules";

fn shared_rules() -> Vec<Rule> {
    let helpers = read_template("_helpers.yaml");
    let start = helpers
        .find(&format!("{{{{- define \"{RULES_HELPER}\" -}}}}"))
        .expect("the rules helper is defined");
    let body = &helpers[start..];
    let end = body.find("{{- end -}}").expect("the rules helper ends");
    serde_yaml::from_str(&strip_template(&body[..end])).expect("parse the shared rules")
}

fn granted() -> BTreeMap<(String, String), BTreeSet<String>> {
    let mut granted: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for rule in shared_rules() {
        for group in &rule.api_groups {
            for resource in &rule.resources {
                granted
                    .entry((group.clone(), resource.clone()))
                    .or_default()
                    .extend(rule.verbs.iter().cloned());
            }
        }
    }
    granted
}

fn verbs(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn the_cluster_role_grants_exactly_what_the_operator_calls() {
    let applied_and_deleted = verbs(&["create", "patch", "delete"]);
    let custom_resource = verbs(&["get", "list", "watch", "update", "patch"]);

    let expected: BTreeMap<(String, String), BTreeSet<String>> = [
        (("", "secrets"), applied_and_deleted.clone()),
        (("", "services"), applied_and_deleted.clone()),
        (("apps", "deployments"), applied_and_deleted.clone()),
        (("batch", "jobs"), applied_and_deleted),
        (
            ("ferriskey.rs", "ferriskeyclusters"),
            custom_resource.clone(),
        ),
        (
            ("ferriskey.rs", "ferriskeyclusters/status"),
            custom_resource.clone(),
        ),
        (
            ("ferriskey.rs", "ferriskeyclusters/finalizers"),
            custom_resource,
        ),
    ]
    .into_iter()
    .map(|((group, resource), verbs)| ((group.to_string(), resource.to_string()), verbs))
    .collect();

    assert_eq!(granted(), expected);
}

#[test]
fn the_operator_cannot_read_secrets_cluster_wide() {
    let secrets = granted()
        .remove(&(String::new(), "secrets".to_string()))
        .expect("the operator manages one secret");

    for verb in ["get", "list", "watch", "update"] {
        assert!(!secrets.contains(verb), "{verb} on secrets");
    }
}

#[test]
fn both_role_kinds_grant_the_shared_rules_and_nothing_else() {
    for template in ["operator/cluster-role.yaml", "operator/role.yaml"] {
        let source = read_template(template);
        assert!(
            source.contains(&format!("include \"{RULES_HELPER}\"")),
            "{template} includes the shared rules"
        );
        assert!(
            !source.contains("apiGroups"),
            "{template} declares no rule of its own"
        );
    }
}

#[test]
fn cluster_wide_rbac_is_only_rendered_without_watched_namespaces() {
    for template in [
        "operator/cluster-role.yaml",
        "operator/cluster-role-binding.yaml",
    ] {
        let source = read_template(template);
        assert!(
            source.contains("(not .Values.operator.watchNamespaces)"),
            "{template} is skipped when namespaces are watched"
        );
    }
    for template in ["operator/role.yaml", "operator/role-binding.yaml"] {
        let source = read_template(template);
        assert!(
            source.contains("range .Values.operator.watchNamespaces"),
            "{template} is rendered once per watched namespace"
        );
    }
}

#[test]
fn watched_namespaces_are_quoted_so_that_yaml_keeps_them_as_strings() {
    for template in ["operator/role.yaml", "operator/role-binding.yaml"] {
        let source = read_template(template);
        assert!(
            source.contains("namespace: {{ . | quote }}"),
            "{template} quotes the watched namespace"
        );
    }
}

#[test]
fn the_chart_refuses_an_environment_entry_that_redefines_the_watched_namespaces() {
    let source = read_template("validate.yaml");
    assert!(
        source.contains("concat .Values.common.env .Values.operator.env"),
        "both environment lists are checked"
    );
    assert!(
        source.contains("\"WATCH_NAMESPACES\""),
        "the watched namespaces variable is reserved"
    );
}

#[test]
fn the_chart_refuses_a_namespace_watched_twice() {
    let source = read_template("validate.yaml");
    assert!(
        source.contains("uniq .Values.operator.watchNamespaces"),
        "duplicate namespaces would render the same Role twice"
    );
}
