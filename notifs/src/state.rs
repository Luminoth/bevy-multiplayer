use std::sync::Arc;

use crate::options::Options;

#[derive(Debug, Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub options: Arc<Options>,
}

impl AppState {
    pub fn new(options: Options) -> Self {
        Self {
            options: Arc::new(options),
        }
    }
}
