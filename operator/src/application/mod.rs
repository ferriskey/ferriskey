use std::sync::Arc;

use kube::Client;
use tracing::{debug, error, info};

use crate::{
    application::cluster::controller::run_cluster_controller,
    domain::{common::services::Service, error::OperatorError},
    infrastructure::cluster::repositories::k8s::K8sClusterRepository,
};

pub mod cluster;

pub type OperatorService = Service<K8sClusterRepository>;
pub struct OperatorApp;

pub async fn create_service() -> Result<OperatorService, OperatorError> {
    let client = Client::try_default()
        .await
        .map_err(|e| OperatorError::InternalServerError {
            message: e.to_string(),
        })?;

    let cluster_repository = K8sClusterRepository::new(client);

    Ok(Service::new(cluster_repository))
}

impl OperatorApp {
    pub async fn run() -> Result<(), OperatorError> {
        debug!("initializing kubernetes client...");
        let client = Client::try_default().await.map_err(|e| {
            error!("unable to create the Kubernetes client: {:?}", e);
            OperatorError::InternalServerError {
                message: format!("Kubernetes client error: {}", e),
            }
        })?;

        info!("kubernetes client initialized");

        let service = create_service().await?;
        let service = Arc::new(service);
        info!("service initialized");

        let cluster_controller = run_cluster_controller(client.clone(), service.clone());

        info!("cluster controller started");

        // Au lieu de join!, utilisons select! pour pouvoir ajouter des logs
        tokio::select! {
            _ = cluster_controller => {
                info!("Cluster controller has stopped.");
            }
        }

        Ok(())
    }
}
