pub mod axum;
pub mod gameserver;
pub mod notifs;
pub mod redis;

pub const GAMESERVER_NOTIFS_CHANNEL: &str = "gameserver:notifs";
pub const GAMECLIENT_NOTIFS_CHANNEL: &str = "gameclient:notifs";
