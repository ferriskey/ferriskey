use crate::btreemap;
use crate::crd::cluster::FerriskeyCluster;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{Container, ContainerPort, EnvVar, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kube::api::{DeleteParams, PostParams};
use kube::{Api, Client, Resource, ResourceExt};
use tracing::info;

pub async fn reconcile_frontend(
    cluster: &FerriskeyCluster,
    client: &Client,
) -> Result<(), kube::Error> {
    let ns = cluster.metadata.namespace.as_deref().unwrap_or("default");
    let name = format!("{}-front", cluster.name_any());
    let api: Api<Deployment> = Api::namespaced(client.clone(), ns);

    if cluster.meta().deletion_timestamp.is_some() {
        if api.get_opt(&name).await?.is_some() {
            api.delete(&name, &DeleteParams::default()).await.ok();
            info!("🗑️ Frontend '{}' supprimé", name);
        }

        return Ok(());
    }

    let labels = btreemap! {
        "app".to_string() => cluster.name_any(),
        "component".to_string() => "front".to_string()
    };

    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(name.clone()),
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
                    labels: Some(labels.clone()),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "frontend".into(),
                        image: Some("ghcr.io/ferriskey/ferriskey-front:latest".into()),
                        image_pull_policy: Some("Always".into()),
                        ports: Some(vec![ContainerPort {
                            container_port: 80,
                            ..Default::default()
                        }]),
                        env: Some(vec![EnvVar {
                            name: "APP_API_URL".into(),
                            value: Some("https://api.ferriskey.bonnal.cloud".into()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    if api.get_opt(&name).await?.is_none() {
        api.create(&PostParams::default(), &deployment).await?;
        info!("🌐 Frontend '{}' déployé", name);
    } else {
        info!("🔁 Frontend '{}' déjà présent", name);
    }

    Ok(())
}
