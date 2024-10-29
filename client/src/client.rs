use bevy::prelude::*;
use bevy_replicon_renet::renet::transport::NetcodeClientTransport;

use game::GameState;

use crate::{camera, connect_server, input, main_menu, ui, AppState, Settings};

#[derive(Debug)]
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            main_menu::MainMenuPlugin,
            connect_server::ConnectServerPlugin,
            camera::FpsCameraPlugin,
            input::InputPlugin,
            ui::UiPlugin,
        ))
        .insert_resource(bevy_replicon_renet::renet::RenetClient::new(
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

fn exit(mut commands: Commands) {
    info!("exit game ...");

    commands.remove_resource::<NetcodeClientTransport>();
}
