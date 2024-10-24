use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServerInfo {
    pub server_id: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_id: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_session_ids: Option<Vec<Uuid>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_player_ids: Option<Vec<String>>,
}
