#![cfg(feature = "server")]

use bevy::prelude::*;

use crate::AppState;

pub fn wait_for_placement(mut game_state: ResMut<NextState<AppState>>) {
    warn!("faking placement!");
    game_state.set(AppState::LoadAssets);
}
