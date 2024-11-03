use bevy::prelude::*;
use bevy_replicon_renet::renet::{
    transport::NetcodeClientTransport, ClientId, ConnectionConfig, RenetClient,
};

use game_common::GameState;

use crate::{camera, connect_server, game, input, main_menu, ui, AppState, Settings};

#[derive(Debug, Default, Resource)]
pub struct ClientState {
    host: Option<String>,
    client_id: Option<ClientId>,
}

impl ClientState {
    #[inline]
    pub fn new_remote(host: impl Into<String>, client_id: ClientId) -> Self {
        Self {
            host: Some(host.into()),
            client_id: Some(client_id),
        }
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        self.host.is_none()
    }

    #[inline]
    pub fn host(&self) -> &Option<String> {
        &self.host
    }

    #[inline]
    pub fn id(&self) -> Option<ClientId> {
        self.client_id
    }
}

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
            game::GamePlugin,
        ))
        .insert_resource(RenetClient::new(ConnectionConfig::default()))
        .init_resource::<Settings>()
        .init_resource::<ClientState>()
        .add_systems(OnEnter(AppState::InGame), enter)
        .add_systems(OnExit(AppState::InGame), exit);
    }
}

fn enter(mut game_state: ResMut<NextState<GameState>>) {
    info!("enter client game ...");

    game_state.set(GameState::LoadAssets);
}

fn exit(mut commands: Commands) {
    info!("exit client game ...");

    commands.remove_resource::<ClientState>();
    commands.remove_resource::<NetcodeClientTransport>();
}

pub fn on_connected_server(client: &ClientState, app_state: &mut NextState<AppState>) {
    if client.is_local() {
        info!("running local");
    } else {
        info!("connected to {:?} as {:?}", client.host(), client.id());
    }

    app_state.set(AppState::InGame);
}
