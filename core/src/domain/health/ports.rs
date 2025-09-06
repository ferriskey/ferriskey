use crate::domain::{
    common::entities::app_errors::CoreError, health::entities::DatabaseHealthStatus,
};

pub trait HealthCheckService: Clone + Send + Sync + 'static {
    fn readness(&self) -> impl Future<Output = Result<DatabaseHealthStatus, CoreError>> + Send;
    fn health(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

pub trait HealthCheckRepository: Clone + Send + Sync + 'static {
    fn health(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
    fn readness(&self) -> impl Future<Output = Result<DatabaseHealthStatus, CoreError>> + Send;
}
