mod controller;
mod crd;
mod macros;

use crate::crd::cluster::FerriskeyCluster;

use crate::controller::cluster::reconcile_cluster;
use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    runtime::controller::{Action, Controller},
};
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

    Controller::new(ferriskey_cluster, Default::default())
        .run(
            |cluster, ctx| async move {
                reconcile_cluster(&cluster, &ctx)
                    .await
                    .map(|_| Action::requeue(std::time::Duration::from_secs(300)))
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
