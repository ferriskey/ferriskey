use std::sync::Arc;

use kube::Client;

use crate::{
    application::services::OperatorService, domain::error::OperatorError,
    infrastructure::cluster::run_cluster_controller,
};

pub mod services;

pub struct OperatorApp;

impl OperatorApp {
    pub async fn run() -> Result<(), OperatorError> {
        let client =
            Client::try_default()
                .await
                .map_err(|e| OperatorError::InternalServerError {
                    message: e.to_string(),
                })?;

        let service = Arc::new(OperatorService::new().await?);

        run_cluster_controller(client.clone(), service.clone()).await;

        Ok(())
    }
}
