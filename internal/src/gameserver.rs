use uuid::Uuid;

pub const GAMESERVER_KEY: &str = "gameserver:{}";
pub const GAMESERVERS_INDEX: &str = "gameservers.index";
pub const WAITING_GAMESERVERS_INDEX: &str = "gameservers:waiting.index";

pub fn get_gameserver_key(server_id: Uuid) -> String {
    format!("gameserver:{}", server_id)
}

pub const GAMESESSION_KEY: &str = "gamesession:{}";
pub const GAMESESSIONS_INDEX: &str = "gamesessions.index";
pub const GAMESESSIONS_BACKFILL_SET: &str = "gamesessions:backfill";

pub fn get_gamesession_key(session_id: Uuid) -> String {
    format!("gamesession:{}", session_id)
}
