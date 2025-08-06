use crate::domain::health::entities::{DatabaseHealthStatus, HealthCheckError};
use crate::domain::health::ports::HealthCheckRepository;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use tokio::time::Instant;

#[derive(Clone)]
pub struct PostgresHealthCheckRepository {
    db: DatabaseConnection,
}

impl PostgresHealthCheckRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl HealthCheckRepository for PostgresHealthCheckRepository {
    async fn check_health(&self) -> Result<u64, HealthCheckError> {
        let start = Instant::now();

        let timeout_duration = tokio::time::Duration::from_secs(2);

        let result = tokio::time::timeout(
            timeout_duration,
            self.db.query_one(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT 1".to_owned(),
            ))
        ).await;

        match result {
            Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u64),
            Ok(Err(e)) => Err(HealthCheckError::DatabaseConnectionError(e.to_string())),
            Err(_) => Err(HealthCheckError::DatabaseConnectionError(
                "Database query timeout after 3 seconds".to_string()
            )),
        }
    }

    async fn check_database_status(&self) -> Result<DatabaseHealthStatus, HealthCheckError> {
        match self.check_health().await {
            Ok(response_time) => {
                let status = if response_time > 1000 {
                    "slow"
                } else {
                    "healthy"
                };

                Ok(DatabaseHealthStatus {
                    status: status.to_string(),
                    response_time_ms: Some(response_time),
                    error: None,
                })
            }
            Err(e) => {
                Err(HealthCheckError::ServiceUnavailable(e.to_string()))
            },
        }
    }
}
