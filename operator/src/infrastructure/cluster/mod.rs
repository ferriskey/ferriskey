use std::sync::Arc;

use futures::StreamExt;
use kube::{Api, Client, ResourceExt};
use kube_runtime::{
    Controller,
    controller::Action,
    finalizer::{Event, finalizer},
    watcher::Config,
};
use tracing::{info, warn};

use crate::{
    application::services::OperatorService,
    domain::{
        cluster::{ClusterService, ClusterSpec},
        error::OperatorError,
    },
    infrastructure::k8s::cluster_crd::FerrisKeyCluster,
};

pub mod repositories;

const FINALIZER: &str = "ferriskey.rs/finalizer";

pub async fn run_cluster_controller(client: Client, service: Arc<OperatorService>) {
    let clusters: Api<FerrisKeyCluster> = Api::all(client.clone());

    Controller::new(clusters, Config::default())
        .run(
            move |obj, _| reconcile(obj, service.clone(), client.clone()),
            error_policy,
            Arc::new(()),
        )
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("✅ Reconciled: {:?}", o),
                Err(e) => warn!("❌ Reconcile failed: {:?}", e),
            }
        })
        .await;
}

async fn reconcile(
    cluster: Arc<FerrisKeyCluster>,
    service: Arc<OperatorService>,
    client: Client,
) -> Result<Action, OperatorError> {
    let ns = cluster.namespace().unwrap_or_else(|| "default".to_string());

    let spec = ClusterSpec {
        name: cluster.name_any(),
        version: cluster.spec.version.clone(),
        replicas: cluster.spec.replicas,
        database_url: cluster.spec.database_url.clone(),
    };

    let action = finalizer(
        &Api::<FerrisKeyCluster>::namespaced(client.clone(), &ns),
        FINALIZER,
        cluster,
        |event| async {
            match event {
                Event::Apply(_obj) => {
                    service.reconcile_cluster(&spec, &ns).await?;

                    Ok::<Action, OperatorError>(Action::requeue(std::time::Duration::from_secs(60)))
                }
                Event::Cleanup(_obj) => {
                    service.cleanup_cluster(&spec, &ns).await?;

                    Ok::<Action, OperatorError>(Action::await_change())
                }
            }
        },
    )
    .await
    .map_err(
        |_: kube_runtime::finalizer::Error<_>| OperatorError::InternalServerError {
            message: "failed to finalizer".into(),
        },
    )?;

    Ok(action)
}

fn error_policy(cluster: Arc<FerrisKeyCluster>, err: &OperatorError, _: Arc<()>) -> Action {
    warn!("error reconciling {:?}: {:?}", cluster.name_any(), err);
    Action::requeue(std::time::Duration::from_secs(20))
}
