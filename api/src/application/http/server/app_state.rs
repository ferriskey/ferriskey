use std::sync::Arc;

use ferriskey_core::application::FerrisKeyService;

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: FerrisKeyService,
}

impl AppState {
    pub fn new(args: Arc<Args>, service: FerrisKeyService) -> Self {
        Self { args, service }
    }
}
