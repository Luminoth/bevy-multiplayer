use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameServerState {
    #[default]
    Init,
    WaitingForPlacement,
    InGame,
    Shutdown,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameServerOrchestration {
    #[default]
    Local,
    Agones,
    GameLift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServerInfo {
    pub addrs: Vec<String>,
    pub port: u16,

    pub state: GameServerState,
    pub orchestration: GameServerOrchestration,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_id: Option<Uuid>,

    pub max_players: usize,

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
