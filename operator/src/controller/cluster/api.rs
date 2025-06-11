use crate::btreemap;
use crate::crd::cluster::FerriskeyCluster;

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Container, ContainerPort, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec,
            SecretKeySelector,
        },
    },
    apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta},
};

use kube::{
    Api, Client, Resource, ResourceExt,
    api::{DeleteParams, PostParams},
};
use tracing::info;

pub async fn reconcile_api(cluster: &FerriskeyCluster, client: &Client) -> Result<(), kube::Error> {
    let ns = cluster.namespace().unwrap();
    let name = cluster.name_any();
    let spec = &cluster.spec;

    let api_name = format!("{name}-api");
    let deploy_api: Api<Deployment> = Api::namespaced(client.clone(), &ns);

    if cluster.meta().deletion_timestamp.is_some() {
        if deploy_api.get_opt(&api_name).await?.is_some() {
            deploy_api
                .delete(&api_name, &DeleteParams::default())
                .await
                .ok();
            info!("🗑️ API '{}' supprimée", api_name);
        }

        return Ok(());
    }

    if deploy_api.get_opt(&api_name).await?.is_some() {
        info!("🔁 Deployment {} déjà existant", api_name);
        return Ok(());
    }

    let backend = &cluster.spec.backend;

    let image = spec
        .backend
        .image
        .clone()
        .unwrap_or_else(|| "ghcr.io/ferriskey/ferriskey-api:latest".into());

    let secret_name = format!("{}-postgres-app", cluster.name_any());
    let secret_key = "uri".to_string();

    let labels = btreemap! {
        "app".into() => api_name.clone(),
        "component".into() => "api".into(),
    };

    let admin_username = backend.username.clone().unwrap_or("admin".into());
    let admin_password = backend.password.clone().unwrap_or("admin".into());
    let admin_email = backend
        .email
        .clone()
        .unwrap_or("admin@ferriskey.dev".into());

    let allowed_origins = backend
        .allowed_origins
        .clone()
        .unwrap_or(vec!["http://localhost:5555".into()])
        .join(",");

    let env_vars = vec![
        EnvVar {
            name: "PORT".into(),
            value: Some("3333".into()),
            ..Default::default()
        },
        EnvVar {
            name: "ENV".into(),
            value: Some("production".into()),
            ..Default::default()
        },
        EnvVar {
            name: "DATABASE_URL".into(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: secret_name,
                    key: secret_key,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        EnvVar {
            name: "ADMIN_USERNAME".into(),
            value: Some(admin_username),
            ..Default::default()
        },
        EnvVar {
            name: "ADMIN_PASSWORD".into(),
            value: Some(admin_password),
            ..Default::default()
        },
        EnvVar {
            name: "ADMIN_EMAIL".into(),
            value: Some(admin_email),
            ..Default::default()
        },
        EnvVar {
            name: "ALLOWED_ORIGINS".into(),
            value: Some(allowed_origins),
            ..Default::default()
        },
    ];

    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(api_name.clone()),
            namespace: Some(ns.clone()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(labels.clone()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    init_containers: Some(vec![Container {
                        name: "migrations".into(),
                        image: Some(image.clone()),
                        command: Some(vec!["/bin/sh".into(), "-c".into()]),
                        args: Some(vec!["sqlx migrate run && echo '✅ done'".into()]),
                        env: Some(env_vars.clone()),
                        ..Default::default()
                    }]),
                    containers: vec![Container {
                        name: "ferriskey".into(),
                        image: Some(image),
                        image_pull_policy: Some("Always".into()),
                        ports: Some(vec![ContainerPort {
                            container_port: 8080,
                            ..Default::default()
                        }]),
                        env: Some(env_vars),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    deploy_api
        .create(&PostParams::default(), &deployment)
        .await?;
    info!("🚀 Déployé : {}", api_name);
    Ok(())
}
