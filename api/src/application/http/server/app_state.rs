use ferriskey_core::application::common::factories::UseCaseBundle;
use ferriskey_core::application::common::services::ServiceBundle;
use std::sync::Arc;

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service_bundle: ServiceBundle,
    pub use_case_bundle: UseCaseBundle,
}

impl AppState {
    pub fn new(
        args: Arc<Args>,
        service_bundle: ServiceBundle,
        use_case_bundle: UseCaseBundle,
    ) -> Self {
        Self {
            args,
            service_bundle,
            use_case_bundle,
        }
    }
}
