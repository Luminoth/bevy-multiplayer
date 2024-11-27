use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::{
    gameserver::{GameServerOrchestration, GameServerState},
    user::UserId,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameServerInfo {
    pub server_id: Uuid,
    pub addrs: Vec<String>,
    pub port: u16,
    pub state: GameServerState,
    pub orchestration: GameServerOrchestration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_id: Option<Uuid>,
}

impl GameServerInfo {
    #[inline]
    pub fn new(server_id: Uuid, server_info: common::gameserver::GameServerInfo) -> Self {
        Self {
            server_id,
            addrs: server_info.addrs,
            port: server_info.port,
            state: server_info.state,
            orchestration: server_info.orchestration,
            game_session_id: server_info.game_session_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSessionInfo {
    pub game_session_id: Uuid,
    pub server_id: Uuid,

    pub max_players: u16,
    pub active_player_ids: Vec<UserId>,
    pub pending_player_ids: Vec<UserId>,
}

impl GameSessionInfo {
    #[inline]
    pub fn new(
        server_id: Uuid,
        server_info: common::gameserver::GameServerInfo,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            game_session_id: server_info
                .game_session_id
                .ok_or_else(|| anyhow::anyhow!("missing game session id"))?,
            server_id,
            max_players: server_info.max_players,
            active_player_ids: server_info
                .active_player_ids
                .ok_or_else(|| anyhow::anyhow!("missing active players"))?,
            pending_player_ids: server_info
                .pending_player_ids
                .ok_or_else(|| anyhow::anyhow!("missing pending players"))?,
        })
    }

    #[inline]
    pub fn player_slots_remaining(&self) -> u16 {
        // TODO: this isn't safe if we mess up and have more players than max_players
        let used_slots = self.active_player_ids.len() + self.pending_player_ids.len();
        self.max_players - used_slots as u16
    }
}
