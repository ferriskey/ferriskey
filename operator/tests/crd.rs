use std::path::PathBuf;

use ferriskey_operator::infrastructure::cluster::crds::FerrisKeyCluster;
use kube::CustomResourceExt;

fn chart_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../charts/ferriskey-operator")
        .join(relative)
}

fn without_template_lines(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("{{"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn generated_crd_yaml() -> String {
    serde_yaml::to_string(&FerrisKeyCluster::crd()).expect("serialize the crd")
}

#[test]
fn unquoted_descriptions_never_contain_a_mapping_indicator() {
    for line in generated_crd_yaml().lines() {
        let Some(value) = line.trim_start().strip_prefix("description: ") else {
            continue;
        };
        if value.starts_with('\'') || value.starts_with('"') {
            continue;
        }
        assert!(
            !value.contains(": ") && !value.contains(" #"),
            "plain scalar that strict yaml parsers refuse: {line}"
        );
    }
}

#[test]
fn the_committed_crd_file_is_the_generated_one() {
    let committed = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("crds/crd-ferriskeycluster.yaml"),
    )
    .expect("read the committed crd");

    assert_eq!(committed, generated_crd_yaml());
}

#[test]
fn the_chart_ships_the_generated_crd_spec() {
    let template =
        std::fs::read_to_string(chart_path("templates/crd.yaml")).expect("read the chart crd");
    let chart: serde_yaml::Value =
        serde_yaml::from_str(&without_template_lines(&template)).expect("parse the chart crd");
    let chart_spec = serde_json::to_value(&chart["spec"]).expect("chart spec as json");

    let generated_spec = serde_json::to_value(FerrisKeyCluster::crd().spec).expect("crd spec");

    assert_eq!(chart_spec, generated_spec);
}
