use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::options::Options;

pub type GameServerSet = Arc<RwLock<HashMap<Uuid, crate::notifs::NotifSender>>>;

#[derive(Debug, Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub options: Arc<Options>,

    pub game_servers: GameServerSet,
}

impl AppState {
    pub fn new(options: Options) -> Self {
        Self {
            options: Arc::new(options),

            game_servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
