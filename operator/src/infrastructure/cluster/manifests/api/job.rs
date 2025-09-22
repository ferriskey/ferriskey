use std::collections::BTreeMap;

use k8s_openapi::api::{
    batch::v1::{Job, JobSpec},
    core::v1::{Container, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec, SecretKeySelector},
};
use kube::api::ObjectMeta;

use crate::domain::cluster::ClusterSpec;

pub fn make_migration_job(spec: &ClusterSpec, namespace: &str) -> Job {
    let app_label = format!("ferriskey-{}", spec.name);
    let job_name = format!("ferriskey-migrations-{}", spec.name);
    let db_secret_ref = spec.database.secret_ref.name.clone();

    let env_vars = vec![EnvVar {
        name: "DATABASE_URL".to_string(),
        value: None,
        value_from: Some(EnvVarSource {
            secret_key_ref: Some(SecretKeySelector {
                name: db_secret_ref,
                key: "uri".to_string(),
                optional: Some(false),
            }),
            ..Default::default()
        }),
    }];

    Job {
        metadata: ObjectMeta {
            name: Some(job_name.clone()),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                ("app".to_string(), app_label.clone()),
                ("component".to_string(), "migration".to_string()),
            ])),
            ..Default::default()
        },
        spec: Some(JobSpec {
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([
                        ("app".to_string(), app_label.clone()),
                        ("component".to_string(), "migration".to_string()),
                    ])),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    restart_policy: Some("Never".to_string()),
                    containers: vec![Container {
                        name: "migration".into(),
                        image: Some(format!("ghcr.io/ferriskey/ferriskey-api:{}", spec.version)),
                        command: Some(vec![
                            "sqlx".to_string(),
                            "migrate".to_string(),
                            "run".to_string(),
                        ]),
                        args: Some(vec![
                            "--source".to_string(),
                            "/usr/local/src/ferriskey/migrations".to_string(),
                        ]),
                        env: Some(env_vars),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            backoff_limit: Some(3),
            ttl_seconds_after_finished: Some(300),
            ..Default::default()
        }),
        status: None,
    }
}
