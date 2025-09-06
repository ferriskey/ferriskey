use crate::domain::common::entities::{InitializationResult, StartupConfig, app_errors::CoreError};

pub trait CoreService: Clone + Send + Sync {
    fn initialize_application(
        &self,
        config: StartupConfig,
    ) -> impl Future<Output = Result<InitializationResult, CoreError>> + Send;
}
