use crate::domain::common::entities::{InitializationResult, StartupConfig, app_errors::CoreError};

pub trait CoreService: Clone + Send + Sync {
    fn initialize_application(
        &self,
        config: StartupConfig,
    ) -> impl Future<Output = Result<InitializationResult, CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        CoreService {}
        impl Clone for CoreService {
            fn clone(&self) -> Self;
        }
        impl CoreService for CoreService {
            fn initialize_application(&self,config: StartupConfig) -> impl Future<Output = Result<InitializationResult, CoreError>> + Send;
        }
    }
}