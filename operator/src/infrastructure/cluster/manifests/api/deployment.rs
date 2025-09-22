use std::{collections::BTreeMap, vec};

use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{Container, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec, SecretKeySelector},
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::api::ObjectMeta;

use crate::domain::cluster::ClusterSpec;

pub fn make_deployment(spec: &ClusterSpec, namespace: &str) -> Deployment {
    let app_label = format!("ferriskey-api-{}", spec.name);
    let admin_secret_name = format!("ferriskey-admin-{}", spec.name);
    let db_secret_ref = spec.database.secret_ref.name.clone();

    let env_vars = vec![
        EnvVar {
            name: "DATABASE_HOST".to_string(),
            value: None,
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: db_secret_ref.clone(),
                    key: "host".to_string(),
                    optional: Some(false),
                }),
                ..Default::default()
            }),
        },
        EnvVar {
            name: "DATABASE_NAME".to_string(),
            value: None,
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: db_secret_ref.clone(),
                    key: "dbname".to_string(),
                    optional: Some(false),
                }),
                ..Default::default()
            }),
        },
        EnvVar {
            name: "DATABASE_PASSWORD".to_string(),
            value: None,
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: db_secret_ref.clone(),
                    key: "password".to_string(),
                    optional: Some(false),
                }),
                ..Default::default()
            }),
        },
        EnvVar {
            name: "DATABASE_PORT".to_string(),
            value: None,
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: db_secret_ref.clone(),
                    key: "port".to_string(),
                    optional: Some(false),
                }),
                ..Default::default()
            }),
        },
        EnvVar {
            name: "DATABASE_USER".to_string(),
            value: None,
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: db_secret_ref.clone(),
                    key: "user".to_string(),
                    optional: Some(false),
                }),
                ..Default::default()
            }),
        },
        EnvVar {
            name: "ADMIN_PASSWORD".to_string(),
            value: None,
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: admin_secret_name,
                    key: "password".to_string(),
                    optional: Some(false),
                }),
                ..Default::default()
            }),
        },
        EnvVar {
            name: "ADMIN_EMAIL".to_string(),
            value: Some("admin@gmail.com".into()),
            ..Default::default()
        },
        EnvVar {
            name: "ADMIN_USERNAME".to_string(),
            value: Some("admin".into()),
            ..Default::default()
        },
        EnvVar {
            name: "ENV".to_string(),
            value: Some("production".into()),
            ..Default::default()
        },
        EnvVar {
            name: "LOG_FILTER".to_string(),
            value: Some("info".to_string()),
            ..Default::default()
        },
        EnvVar {
            name: "LOG_JSON".to_string(),
            value: Some("true".to_string()),
            ..Default::default()
        },
        EnvVar {
            name: "SERVER_PORT".to_string(),
            value: Some("3333".to_string()),
            ..Default::default()
        },
        EnvVar {
            name: "SERVER_ROOT_PATH".to_string(),
            value: Some("/".to_string()),
            ..Default::default()
        },
        EnvVar {
            name: "WEBAPP_URL".to_string(),
            value: Some(spec.api.webapp_url.clone()),
            ..Default::default()
        },
        EnvVar {
            name: "ALLOWED_ORIGINS".to_string(),
            value: Some(spec.api.allowed_origins.join(",")),
            ..Default::default()
        },
    ];

    Deployment {
        metadata: ObjectMeta {
            name: Some(format!("ferriskey-api-{}", spec.name)),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                ("app".to_string(), app_label.clone()),
                ("component".to_string(), "api".to_string()),
            ])),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: Some(spec.replicas as i32),
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([("app".to_string(), app_label.clone())])),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([(
                        "app".to_string(),
                        app_label.clone(), // ← Correction ici : utilise le même label
                    )])),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "ferriskey-api".into(),
                        image: Some(format!("ghcr.io/ferriskey/ferriskey-api:{}", spec.version)),
                        env: Some(env_vars),
                        ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                            container_port: 3333,
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        status: None,
    }
}
