use std::sync::Arc;

use crate::{options::Options, redis::RedisConnectionPool};

#[derive(Debug, Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub options: Arc<Options>,

    pub redis_connection_pool: RedisConnectionPool,
}

impl AppState {
    pub fn new(options: Options, redis_connection_pool: RedisConnectionPool) -> Self {
        Self {
            options: Arc::new(options),
            redis_connection_pool,
        }
    }
}
