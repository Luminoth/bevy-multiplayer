use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::gameserver::GameServerState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServerInfo {
    pub server_id: Uuid,
    pub addrs: Vec<String>,
    pub port: u16,
    pub state: GameServerState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_id: Option<Uuid>,
}

impl From<common::gameserver::GameServerInfo> for GameServerInfo {
    fn from(server_info: common::gameserver::GameServerInfo) -> Self {
        Self {
            server_id: server_info.server_id,
            addrs: server_info.addrs,
            port: server_info.port,
            state: server_info.state,
            game_session_id: server_info.game_session_id,
        }
    }
}

impl GameServerInfo {
    pub fn get_key(&self) -> String {
        format!("gameserver:{}", self.server_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSessionInfo {
    pub game_session_id: Uuid,
    pub server_id: Uuid,

    pub player_session_ids: Vec<Uuid>,
    pub pending_player_ids: Vec<String>,
}

impl GameSessionInfo {
    pub fn get_key(&self) -> String {
        format!("gamesession:{}", self.game_session_id)
    }
}

impl TryFrom<common::gameserver::GameServerInfo> for GameSessionInfo {
    type Error = anyhow::Error;

    fn try_from(server_info: common::gameserver::GameServerInfo) -> anyhow::Result<Self> {
        Ok(Self {
            game_session_id: server_info
                .game_session_id
                .ok_or_else(|| anyhow::anyhow!("missing game session id"))?,
            server_id: server_info.server_id,
            player_session_ids: server_info
                .player_session_ids
                .ok_or_else(|| anyhow::anyhow!("missing player session ids"))?,
            pending_player_ids: server_info
                .pending_player_ids
                .ok_or_else(|| anyhow::anyhow!("missing pending players"))?,
        })
    }
}
