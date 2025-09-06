use crate::{
    application::common::FerriskeyService,
    domain::{
        common::entities::app_errors::CoreError,
        health::{
            entities::DatabaseHealthStatus,
            ports::{HealthCheckRepository, HealthCheckService},
        },
    },
};

impl HealthCheckService for FerriskeyService {
    async fn readness(&self) -> Result<DatabaseHealthStatus, CoreError> {
        self.health_check_repository.readness().await
    }

    async fn health(&self) -> Result<u64, CoreError> {
        self.health_check_repository.health().await
    }
}
