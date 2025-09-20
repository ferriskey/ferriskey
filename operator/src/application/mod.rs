use std::sync::Arc;

use kube::Client;
use tracing::{debug, error, info};

use crate::{
    application::services::OperatorService, domain::error::OperatorError,
    infrastructure::cluster::run_cluster_controller,
};

pub mod services;

pub struct OperatorApp;

impl OperatorApp {
    pub async fn run() -> Result<(), OperatorError> {
        debug!("🔧 Tentative de connexion au cluster Kubernetes...");

        let client = Client::try_default().await.map_err(|e| {
            error!("❌ Impossible de se connecter au cluster Kubernetes: {}", e);
            OperatorError::InternalServerError {
                message: format!("Kubernetes client error: {}", e),
            }
        })?;

        info!("✅ Client Kubernetes initialisé avec succès");

        debug!("🔧 Initialisation du service opérateur...");
        let service = Arc::new(OperatorService::new().await?);
        info!("✅ Service opérateur initialisé");

        debug!("🔧 Démarrage du contrôleur de cluster...");
        let cluster_controller = run_cluster_controller(client.clone(), service.clone());

        info!("✅ Contrôleur de cluster démarré");

        // Au lieu de join!, utilisons select! pour pouvoir ajouter des logs
        tokio::select! {
            _ = cluster_controller => {
                info!("🔄 Contrôleur de cluster terminé");
            }
        }

        Ok(())
    }
}
