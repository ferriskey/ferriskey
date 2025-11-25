use crate::{
    ApplicationService,
    domain::{
        common::entities::app_errors::CoreError,
        health::{entities::DatabaseHealthStatus, ports::HealthCheckService},
    },
};

impl HealthCheckService for ApplicationService {
    async fn health(&self) -> Result<u64, CoreError> {
        self.health_service.health().await
    }

    async fn readness(&self) -> Result<DatabaseHealthStatus, CoreError> {
        self.health_service.readness().await
    }
}
