use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct Role {
    rules: Vec<Rule>,
}

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

fn granted() -> BTreeMap<(String, String), BTreeSet<String>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../charts/ferriskey-operator/templates/operator/cluster-role.yaml");
    let source = std::fs::read_to_string(path).expect("read the cluster role template");
    let role: Role =
        serde_yaml::from_str(&strip_template(&source)).expect("parse the cluster role");

    let mut granted: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for rule in role.rules {
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
