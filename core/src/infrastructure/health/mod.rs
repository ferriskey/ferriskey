use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::health::entities::DatabaseHealthStatus;
use crate::domain::health::ports::HealthCheckRepository;
use crate::infrastructure::health::repositories::PostgresHealthCheckRepository;

pub mod repositories;

#[derive(Clone)]
pub enum HealthCheckRepoAny {
    Postgres(PostgresHealthCheckRepository),
}

impl HealthCheckRepository for HealthCheckRepoAny {
    async fn health(&self) -> Result<u64, CoreError> {
        match self {
            HealthCheckRepoAny::Postgres(r) => r.health().await,
        }
    }

    async fn readness(&self) -> Result<DatabaseHealthStatus, CoreError> {
        match self {
            HealthCheckRepoAny::Postgres(r) => r.readness().await,
        }
    }
}
