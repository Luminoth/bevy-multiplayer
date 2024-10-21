#![cfg(feature = "server")]

use bevy::prelude::*;

use crate::AppState;

pub fn wait_for_placement(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::LoadAssets);
}
