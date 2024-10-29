use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum GameServerState {
    Init,
    WaitingForPlacement,
    InGame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServerInfo {
    pub server_id: Uuid,

    pub state: GameServerState,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_id: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_session_ids: Option<Vec<Uuid>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_player_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostHeartbeatRequestV1 {
    pub server_info: GameServerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostHeartbeatResponseV1 {}
