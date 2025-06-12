use crate::crd::cluster::FerriskeyCluster;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Resource, ResourceExt};
use serde_json::json;

pub mod api;
pub mod frontend;
pub mod postgres;

fn build_owner_reference(cluster: &FerriskeyCluster) -> OwnerReference {
    OwnerReference {
        api_version: "ferriskey.io/v1".to_string(),
        kind: "FerriskeyCluster".to_string(),
        name: cluster.name_any(),
        uid: cluster.meta().uid.clone().unwrap_or_default(),
        controller: Some(true),
        block_owner_deletion: Some(true),
    }
}

async fn ensure_finalizer(cluster: &FerriskeyCluster, client: &Client) -> Result<(), kube::Error> {
    if cluster
        .finalizers()
        .contains(&"ferriskey.io/finalizer".to_string())
    {
        return Ok(());
    }

    let ns = cluster.namespace().unwrap_or("default".into());
    let api: Api<FerriskeyCluster> = Api::namespaced(client.clone(), &ns);

    let patch = json!({
        "metadata": {
            "finalizers": [
                "ferriskey.io/finalizer"
            ]
        }
    });

    api.patch(
        &cluster.name_any(),
        &PatchParams::apply("ferriskey-operator"),
        &Patch::Merge(&patch),
    )
    .await?;
    Ok(())
}

async fn remove_finalizer(cluster: &FerriskeyCluster, client: &Client) -> Result<(), kube::Error> {
    let finalizers: Vec<String> = cluster
        .finalizers()
        .iter()
        .filter(|f| *f != "ferriskey.io/finalizer")
        .cloned()
        .collect();

    let ns = cluster.namespace().unwrap_or("default".to_string());
    let api: Api<FerriskeyCluster> = Api::namespaced(client.clone(), &ns);

    let patch = json!({
        "metadata": {
            "finalizers": finalizers
        }
    });

    api.patch(
        &cluster.name_any(),
        &PatchParams::default(),
        &Patch::Merge(&patch),
    )
    .await?;
    Ok(())
}

pub async fn reconcile_cluster(
    cluster: &FerriskeyCluster,
    client: &Client,
) -> Result<(), kube::Error> {
    ensure_finalizer(cluster, client).await?;

    if cluster.meta().deletion_timestamp.is_some() {
        postgres::reconcile_postgres(cluster, client).await?;

        api::reconcile_api(cluster, client).await?;
        api::reconcile_api_service(cluster, client).await?;

        frontend::reconcile_frontend(cluster, client).await?;
        frontend::reconcile_frontend_service(cluster, client).await?;

        remove_finalizer(cluster, client).await?;
        return Ok(());
    }

    postgres::reconcile_postgres(cluster, client).await?;

    api::reconcile_api(cluster, client).await?;
    api::reconcile_api_service(cluster, client).await?;

    frontend::reconcile_frontend(cluster, client).await?;
    frontend::reconcile_frontend_service(cluster, client).await?;

    Ok(())
}
