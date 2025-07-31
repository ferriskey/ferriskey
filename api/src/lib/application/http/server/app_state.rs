use std::sync::Arc;
use ferriskey_core::application::common::factories::{UseCaseBundle};
use ferriskey_core::infrastructure::common::factories::service_factory::ServiceBundle;
use crate::{
    env::Env,
};

#[derive(Clone)]
pub struct AppState {
    pub env: Arc<Env>,

    pub service_bundle: ServiceBundle,
    pub use_case_bundle: UseCaseBundle,
}

impl AppState {
    pub fn new(env: Arc<Env>, service_bundle: ServiceBundle, use_case_bundle: UseCaseBundle) -> Self {
        Self {
            env,
            service_bundle,
            use_case_bundle,
        }
    }
}