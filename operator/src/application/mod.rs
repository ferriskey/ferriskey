use std::sync::Arc;

use kube::Client;
use tracing::{debug, error, info};

use crate::{
    application::{cluster::controller::run_cluster_controller, services::OperatorService},
    domain::{common::OperatorConfig, error::OperatorError},
};

pub mod cluster;
pub mod services;

pub struct OperatorApp;

impl OperatorApp {
    pub async fn run(config: &OperatorConfig) -> Result<(), OperatorError> {
        debug!("initializing kubernetes client...");
        let client = Client::try_default().await.map_err(|e| {
            error!("unable to create the Kubernetes client: {:?}", e);
            OperatorError::InternalServerError {
                message: format!("Kubernetes client error: {}", e),
            }
        })?;

        info!("kubernetes client initialized");

        let service = Arc::new(OperatorService::new(config).await?);
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
