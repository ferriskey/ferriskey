use std::sync::Arc;

use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    api::{Patch, PatchParams},
};
use kube_runtime::{
    Controller,
    controller::Action,
    finalizer::{Event, finalizer},
    watcher::Config,
};
use serde_json::json;
use tracing::{info, warn};

use crate::{
    application::services::OperatorService,
    domain::{
        cluster::{ApiSpec, ClusterService, ClusterSpec, DatabaseConfig, SecretReference},
        error::OperatorError,
    },
    infrastructure::cluster::crds::{FerrisKeyCluster, FerrisKeyClusterStatus},
};

pub mod crds;
pub mod manifests;
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
                Ok(o) => info!("Reconciled: {:?}", o),
                Err(e) => warn!("Reconcile failed: {:?}", e),
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
    let api: Api<FerrisKeyCluster> = Api::namespaced(client.clone(), &ns);

    let secret_ref = SecretReference {
        name: cluster.spec.database.secret_ref.name.clone(),
        namespace: cluster.spec.database.secret_ref.namespace.clone(),
    };
    let database = DatabaseConfig {
        database_name: cluster.spec.database.database_name.clone(),
        secret_ref,
        ssl_mode: cluster.spec.database.ssl_mode.clone(),
    };

    let api_spec = ApiSpec {
        allowed_origins: cluster.spec.api.allowed_origins.clone(),
        api_url: cluster.spec.api.api_url.clone(),
        webapp_url: cluster.spec.api.webapp_url.clone(),
    };

    let spec = ClusterSpec {
        name: cluster.name_any(),
        version: cluster.spec.version.clone(),
        replicas: cluster.spec.replicas,
        database,

        api: api_spec,
    };

    let action = finalizer(&api, FINALIZER, cluster, |event| async {
        match event {
            Event::Apply(obj) => {
                match service.reconcile_cluster(&spec, &ns).await {
                    Ok(_) => {
                        if let Err(e) = update_status(
                            &api,
                            &obj.name_any(),
                            FerrisKeyClusterStatus {
                                ready: true,
                                message: Some("Cluster successfully deployed".to_string()),
                                conditions: None,
                                database_status: None,
                                phase: Some("Running".to_string()),
                            },
                        )
                        .await
                        {
                            warn!("failed to update status: {:?}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(status_err) = update_status(
                            &api,
                            &obj.name_any(),
                            FerrisKeyClusterStatus {
                                ready: false,
                                message: Some(format!("Error deploying cluster: {}", e)),
                                conditions: None,
                                database_status: None,
                                phase: Some("Error".to_string()),
                            },
                        )
                        .await
                        {
                            warn!("failed to update status: {:?}", status_err);
                        }
                        return Err(e);
                    }
                }

                Ok::<Action, OperatorError>(Action::requeue(std::time::Duration::from_secs(60)))
            }
            Event::Cleanup(_) => {
                info!("starting cleanup for cluster: {}", spec.name);
                service.cleanup_cluster(&spec, &ns).await?;

                info!("cleanup completed for cluster: {}", spec.name);

                Ok::<Action, OperatorError>(Action::await_change())
            }
        }
    })
    .await;

    // Gérer spécifiquement les erreurs de finalizer
    match action {
        Ok(action) => Ok(action),
        Err(e) => match &e {
            kube_runtime::finalizer::Error::RemoveFinalizer(kube::Error::Api(api_err))
                if api_err.code == 404 =>
            {
                info!("Resource already deleted, finalizer removal completed");
                Ok(Action::await_change())
            }
            _ => {
                tracing::error!("❌ Erreur du finalizer: {:?}", e);
                Err(OperatorError::InternalServerError {
                    message: format!("Finalizer error: {:?}", e),
                })
            }
        },
    }
}

async fn update_status(
    api: &Api<FerrisKeyCluster>,
    name: &str,
    status: FerrisKeyClusterStatus,
) -> Result<(), OperatorError> {
    let mut cluster = api
        .get(name)
        .await
        .map_err(|e| OperatorError::InternalServerError {
            message: format!("Failed to get cluster for status update: {}", e),
        })?;

    cluster.status = Some(status.clone());

    let status_value =
        serde_json::to_value(&status).map_err(|e| OperatorError::InternalServerError {
            message: e.to_string(),
        })?;

    let patch_value = json!({
        "status": status_value
    });

    let patch = Patch::Merge(&patch_value);
    let pp = PatchParams::default();

    api.patch_status(name, &pp, &patch)
        .await
        .map_err(|e| OperatorError::InternalServerError {
            message: format!("Failed to patch cluster status: {}", e),
        })?;

    info!("status updated for cluster: {}", name);

    Ok(())
}

fn error_policy(cluster: Arc<FerrisKeyCluster>, err: &OperatorError, _: Arc<()>) -> Action {
    warn!("error reconciling {:?}: {:?}", cluster.name_any(), err);
    Action::requeue(std::time::Duration::from_secs(20))
}
