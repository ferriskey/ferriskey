use ferriskey_core::application::common::FerriskeyService;
use std::sync::Arc;

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: FerriskeyService,
}

impl AppState {
    pub fn new(args: Arc<Args>, service: FerriskeyService) -> Self {
        Self { args, service }
    }
}
