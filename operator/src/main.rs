mod controller;
mod crd;
mod macros;

use crate::crd::cluster::FerriskeyCluster;

use crate::controller::cluster::reconcile_cluster;
use futures::StreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Service;
use kube::api::{ApiResource, DynamicObject, GroupVersionKind};
use kube::{
    Api, Client, ResourceExt,
    runtime::controller::{Action, Controller},
};
use kube_runtime::watcher;
use std::sync::Arc;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    run().await?;
    Ok(())
}

async fn run() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let ferriskey_cluster = Api::<FerriskeyCluster>::all(client.clone());
    let deployments = Api::<Deployment>::all(client.clone());
    let services = Api::<Service>::all(client.clone());

    let cnpg_gvk = GroupVersionKind::gvk("postgresql.cnpg.io", "v1", "Cluster");
    let cnpg_ar = ApiResource::from_gvk(&cnpg_gvk);
    let cnpg_api: Api<DynamicObject> = Api::all_with(client.clone(), &cnpg_ar);

    Controller::new(ferriskey_cluster, watcher::Config::default())
        .owns(deployments, watcher::Config::default())
        .owns(services, watcher::Config::default())
        .owns_with(cnpg_api, cnpg_ar, watcher::Config::default())
        .run(
            |cluster, ctx| async move {
                reconcile_cluster(&cluster, &ctx)
                    .await
                    .map(|_| Action::await_change())
            },
            |cluster, err, _ctx| {
                warn!(
                    "❌ Erreur de reconciliation: {:?} pour {:?}",
                    err,
                    cluster.name_any()
                );
                Action::requeue(std::time::Duration::from_secs(10))
            },
            Arc::new(client),
        )
        .for_each(|res| async move {
            match res {
                Ok((obj_ref, _)) => info!(
                    "✅ Reconciled: {} in namespace {:?}",
                    obj_ref.name, obj_ref.namespace
                ),
                Err(err) => warn!("❌ Reconcile failed: {:?}", err),
            }
        })
        .await;

    Ok(())
}
