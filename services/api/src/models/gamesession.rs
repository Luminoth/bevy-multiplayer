use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::user::UserId;

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
    pub fn new(server_id: Uuid, session_info: &common::gameserver::GameSessionInfo) -> Self {
        Self {
            game_session_id: session_info.game_session_id,
            server_id,
            max_players: session_info.max_players,
            active_player_ids: session_info.active_player_ids.clone(),
            pending_player_ids: session_info.pending_player_ids.clone(),
        }
    }

    #[inline]
    pub fn player_slots_remaining(&self) -> u16 {
        // TODO: this isn't safe if we mess up and have more players than max_players
        let used_slots = self.active_player_ids.len() + self.pending_player_ids.len();
        self.max_players - used_slots as u16
    }
}
