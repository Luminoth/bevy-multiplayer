use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::gameserver::{GameServerOrchestration, GameServerState};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameServerInfo {
    pub server_id: Uuid,

    pub v4addrs: Vec<String>,
    pub v6addrs: Vec<String>,
    pub port: u16,

    pub state: GameServerState,
    pub orchestration: GameServerOrchestration,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_id: Option<Uuid>,
}

impl GameServerInfo {
    #[inline]
    pub fn new(server_id: Uuid, server_info: &common::gameserver::GameServerInfo) -> Self {
        Self {
            server_id,
            v4addrs: server_info.v4addrs.clone(),
            v6addrs: server_info.v6addrs.clone(),
            port: server_info.port,
            state: server_info.state,
            orchestration: server_info.orchestration,
            game_session_id: server_info
                .game_session_info
                .as_ref()
                .map(|game_session_info| game_session_info.game_session_id),
        }
    }
}
