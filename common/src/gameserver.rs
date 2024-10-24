use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameServerInfo {
    pub server_id: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_session_id: Option<Uuid>,
}
