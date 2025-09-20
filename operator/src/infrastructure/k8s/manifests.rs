use std::{collections::BTreeMap, vec};

use k8s_openapi::{
    ByteString,
    api::{
        apps::v1::Deployment,
        batch::v1::{Job, JobSpec},
        core::v1::{
            Container, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec, Secret, SecretKeySelector,
            Service, ServicePort, ServiceSpec,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::api::ObjectMeta;
use rand::seq::{IndexedRandom, SliceRandom};

use crate::domain::cluster::ClusterSpec;

fn generate_password(length: usize) -> String {
    let length = length.max(12);

    // Character sets
    let lowercase = b"abcdefghijklmnopqrstuvwxyz";
    let uppercase = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let digits = b"0123456789";
    let symbols = b"!@#$%^&*()-_=+[]{};:,.<>?";

    let mut rng = rand::rng();

    // Ensure at least one from each category
    let mut password: Vec<u8> = vec![
        *lowercase.choose(&mut rng).unwrap(),
        *uppercase.choose(&mut rng).unwrap(),
        *digits.choose(&mut rng).unwrap(),
        *symbols.choose(&mut rng).unwrap(),
    ];

    // Pool of all allowed characters
    let all: Vec<u8> = lowercase
        .iter()
        .chain(uppercase.iter())
        .chain(digits.iter())
        .chain(symbols.iter())
        .copied()
        .collect();

    // Fill the rest
    for _ in password.len()..length {
        password.push(*all.choose(&mut rng).unwrap());
    }

    // Shuffle so guaranteed chars are not predictable
    password.shuffle(&mut rng);

    String::from_utf8(password).unwrap()
}

pub fn make_admin_secret(spec: &ClusterSpec, namespace: &str) -> Secret {
    let secret_name = format!("ferriskey-admin-{}", spec.name);

    let mut data = BTreeMap::new();

    let random_password = generate_password(16);

    data.insert(
        "password".to_string(),
        ByteString(random_password.as_bytes().to_vec()),
    );

    Secret {
        metadata: ObjectMeta {
            name: Some(secret_name),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                ("app".to_string(), format!("ferriskey-{}", spec.name)),
                ("component".to_string(), "admin-secret".to_string()),
            ])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    }
}

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
            backoff_limit: Some(3), // Retry jusqu'à 3 fois en cas d'échec
            ttl_seconds_after_finished: Some(300), // Supprimer le job 5 minutes après completion
            ..Default::default()
        }),
        status: None,
    }
}

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

pub fn api_service(spec: &ClusterSpec, namespace: &str) -> Service {
    let app_label = format!("ferriskey-api-{}", spec.name);

    Service {
        metadata: ObjectMeta {
            name: Some(format!("ferriskey-api-{}", spec.name)),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(BTreeMap::from([(
                "app".to_string(),
                app_label, // ← Correction ici aussi
            )])),
            ports: Some(vec![ServicePort {
                port: 3333,
                target_port: Some(
                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(3333),
                ),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        status: None,
    }
}
