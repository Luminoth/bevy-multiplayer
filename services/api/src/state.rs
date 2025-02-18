use std::sync::Arc;

use crate::{options::Options, redis::RedisConnection};

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub options: Arc<Options>,

    pub redis_connection: RedisConnection,
}

impl AppState {
    pub fn new(options: Options, redis_connection: RedisConnection) -> Self {
        Self {
            options: Arc::new(options),
            redis_connection,
        }
    }
}
