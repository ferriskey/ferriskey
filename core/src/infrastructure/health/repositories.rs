use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use tokio::time::Instant;
use crate::domain::health::entities::{DatabaseHealthStatus, HealthCheckError};
use crate::domain::health::ports::HealthCheckRepository;

#[derive(Clone)]
pub struct PostgresHealthCheckRepository {
    db: DatabaseConnection
}

impl PostgresHealthCheckRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl HealthCheckRepository for PostgresHealthCheckRepository {
    async fn check_health(&self) -> Result<u64, HealthCheckError> {
        let start = Instant::now();


        let result = self.db
            .query_one(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT 1".to_owned(),
            ))
            .await;

        match result {
            Ok(_) => Ok(start.elapsed().as_millis() as u64),
            Err(e) => Err(HealthCheckError::DatabaseConnectionError(e.to_string())),
        }
    }

    async fn check_database_status(&self) -> Result<DatabaseHealthStatus, HealthCheckError>{

        match self.check_health().await {
            Ok(response_time) => {
                let status = if response_time > 1000 { "slow" } else { "healthy" };

                Ok(DatabaseHealthStatus {
                    status: status.to_string(),
                    response_time_ms: Some(response_time),
                    error: None,
                })
            },
            Err(e) => Ok(DatabaseHealthStatus {
                status: "unhealthy".to_string(),
                response_time_ms: None,
                error: Some(e.to_string()),
            }),
        }
    }
}
