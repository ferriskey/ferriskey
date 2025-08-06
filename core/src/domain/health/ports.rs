use crate::domain::health::entities::{DatabaseHealthStatus, HealthCheckError};

pub trait HealthCheckService: Clone + Send + Sync + 'static {
    fn check_health(&self) -> impl Future<Output = Result<u64, HealthCheckError>> + Send;
    fn check_database_status(&self) -> impl Future<Output = Result<DatabaseHealthStatus, HealthCheckError>> + Send;
}

pub trait HealthCheckRepository: Clone + Send + Sync + 'static {
    fn check_health(&self) -> impl Future<Output = Result<u64, HealthCheckError>> + Send;
    fn check_database_status(&self) -> impl Future<Output = Result<DatabaseHealthStatus, HealthCheckError>> + Send;
}