pub mod gameclient;
pub mod gameserver;
mod gamesettings;
pub mod user;

use bevy_mod_reqwest::*;
use tracing::error;

pub use gamesettings::*;

pub fn check_reqwest_error(response: &ReqwestResponseEvent) -> bool {
    if response.status().is_success() {
        return true;
    }

    error!(
        "got error response {}: {}",
        response.status(),
        response.as_str().unwrap_or("invalid response")
    );

    false
}
