use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::UserId;

// TODO: things not shared with the client should be moved to the internal lib

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameServerState {
    #[default]
    Init,
    WaitingForPlacement,
    Loading,
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
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub addrs: Vec<String>,
    pub port: u16,

    pub state: GameServerState,
    pub orchestration: GameServerOrchestration,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_info: Option<GameSessionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSessionInfo {
    pub game_session_id: Uuid,

    pub max_players: u16,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub active_player_ids: Vec<UserId>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending_player_ids: Vec<UserId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostHeartbeatRequestV1 {
    pub server_info: GameServerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostHeartbeatResponseV1 {}
