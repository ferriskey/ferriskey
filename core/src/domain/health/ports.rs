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

#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;

    mock! {
        pub HealthCheckService {}
        impl Clone for HealthCheckService { fn clone(&self) -> Self; }
        impl HealthCheckService for HealthCheckService {
            fn readness(&self) -> impl Future<Output = Result<DatabaseHealthStatus, CoreError>> + Send;
            fn health(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
        }
    }
    pub fn get_mock_health_check_service_with_clone_expectations() -> MockHealthCheckService {
        let mut mock = MockHealthCheckService::new();
        mock.expect_clone()
            .returning(|| get_mock_health_check_service_with_clone_expectations());
        mock
    }
    mock! {
        pub HealthCheckRepository {}
        impl Clone for HealthCheckRepository { fn clone(&self) -> Self; }
        impl HealthCheckRepository for HealthCheckRepository {
            fn health(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
            fn readness(&self) -> impl Future<Output = Result<DatabaseHealthStatus, CoreError>> + Send;
        }
    }
    pub fn get_mock_health_check_repository_with_clone_expectations() -> MockHealthCheckRepository {
        let mut mock = MockHealthCheckRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_health_check_repository_with_clone_expectations());
        mock
    }
}
