pub mod axum;
pub mod gameserver;
mod gamesettings;
pub mod notifs;
pub mod redis;

pub use gamesettings::*;

pub const GAMESERVER_NOTIFS_CHANNEL: &str = "gameserver:notifs";
