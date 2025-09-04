use ferriskey_core::application::common::{FerriskeyService, factories::UseCaseBundle};
use std::sync::Arc;

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub use_case_bundle: UseCaseBundle,
    pub service: FerriskeyService,
}

impl AppState {
    pub fn new(args: Arc<Args>, use_case_bundle: UseCaseBundle, service: FerriskeyService) -> Self {
        Self {
            args,
            use_case_bundle,
            service,
        }
    }
}
