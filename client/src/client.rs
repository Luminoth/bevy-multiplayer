use bevy::prelude::*;

use game::GameState;

use crate::{AppState, Settings};

#[derive(Debug)]
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy_replicon_renet::renet::RenetClient::new(
            bevy_replicon_renet::renet::ConnectionConfig::default(),
        ))
        .init_resource::<Settings>()
        .add_systems(OnEnter(AppState::InGame), enter)
        .add_systems(OnExit(AppState::InGame), exit);
    }
}

fn enter(mut game_state: ResMut<NextState<GameState>>) {
    info!("enter game ...");

    game_state.set(GameState::LoadAssets);
}

fn exit() {
    info!("exit game ...");
}
