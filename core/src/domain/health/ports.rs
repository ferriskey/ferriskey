use crate::domain::{
    common::entities::app_errors::CoreError, health::entities::DatabaseHealthStatus,
};

pub trait HealthCheckService: Send + Sync {
    fn readness(&self) -> impl Future<Output = Result<DatabaseHealthStatus, CoreError>> + Send;
    fn health(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait HealthCheckRepository: Send + Sync {
    fn health(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
    fn readness(&self) -> impl Future<Output = Result<DatabaseHealthStatus, CoreError>> + Send;
}
