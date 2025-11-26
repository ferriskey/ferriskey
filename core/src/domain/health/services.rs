use std::sync::Arc;

use crate::domain::{
    common::entities::app_errors::CoreError,
    health::{
        entities::DatabaseHealthStatus,
        ports::{HealthCheckRepository, HealthCheckService},
    },
};

#[derive(Clone, Debug)]
pub struct HealthServiceImpl<H>
where
    H: HealthCheckRepository,
{
    pub(crate) health_check_repository: Arc<H>,
}

impl<H> HealthServiceImpl<H>
where
    H: HealthCheckRepository,
{
    pub fn new(health_check_repository: Arc<H>) -> Self {
        Self {
            health_check_repository,
        }
    }
}

impl<H> HealthCheckService for HealthServiceImpl<H>
where
    H: HealthCheckRepository,
{
    async fn readness(&self) -> Result<DatabaseHealthStatus, CoreError> {
        self.health_check_repository.readness().await
    }

    async fn health(&self) -> Result<u64, CoreError> {
        self.health_check_repository.health().await
    }
}
