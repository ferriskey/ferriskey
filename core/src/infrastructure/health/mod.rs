use crate::domain::health::entities::{DatabaseHealthStatus, HealthCheckError};
use crate::domain::health::ports::HealthCheckRepository;
use crate::infrastructure::health::repositories::PostgresHealthCheckRepository;

pub mod repositories;

#[derive(Clone)]
pub enum HealthCheckRepoAny {
    Postgres(PostgresHealthCheckRepository),
}

impl HealthCheckRepository for HealthCheckRepoAny {
    async fn check_health(&self) -> Result<u64, HealthCheckError> {
        match self {
            HealthCheckRepoAny::Postgres(repo) => repo.check_health().await,
        }
    }

    async fn check_database_status(&self) -> Result<DatabaseHealthStatus, HealthCheckError> {
        match self {
            HealthCheckRepoAny::Postgres(repo) => repo.check_database_status().await,
        }
    }
}
