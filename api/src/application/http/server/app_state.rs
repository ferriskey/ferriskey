use std::sync::Arc;

use ferriskey_core::ApplicationService;

use crate::args::Args;

#[derive(Clone, Debug)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: ApplicationService,
}

impl AppState {
    pub fn new(args: Arc<Args>, service: ApplicationService) -> Self {
        Self { args, service }
    }
}
