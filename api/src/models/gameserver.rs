use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServerInfo {
    server_id: Uuid,
    game_session_id: Option<Uuid>,
}

impl From<common::gameserver::GameServerInfo> for GameServerInfo {
    fn from(server_info: common::gameserver::GameServerInfo) -> Self {
        Self {
            server_id: server_info.server_id,
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
    game_session_id: Uuid,
    server_id: Uuid,

    player_session_ids: Vec<Uuid>,
    pending_player_ids: Vec<String>,
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
