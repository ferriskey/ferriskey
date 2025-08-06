use crate::domain::health::entities::{DatabaseHealthStatus, HealthCheckError};
use crate::domain::health::ports::{HealthCheckRepository, HealthCheckService};

#[derive(Clone)]
pub struct HealthCheckServiceImpl<H>
where
    H: HealthCheckRepository,
{
    health_check_repository: H,
}

impl<H> HealthCheckServiceImpl<H>
where
    H: HealthCheckRepository,
{
    pub fn new(health_check_repository: H) -> Self {
        Self {
            health_check_repository,
        }
    }
}

impl<H> HealthCheckService for HealthCheckServiceImpl<H>
where
    H: HealthCheckRepository,
{
    fn check_health(&self) -> impl Future<Output = Result<u64, HealthCheckError>> + Send {
        self.health_check_repository.check_health()
    }

    fn check_database_status(
        &self,
    ) -> impl Future<Output = Result<DatabaseHealthStatus, HealthCheckError>> + Send {
        self.health_check_repository.check_database_status()
    }
}
