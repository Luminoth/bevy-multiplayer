use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::renet::{transport::NetcodeClientTransport, RenetClient};

use game_common::{
    network::{InputUpdateEvent, PlayerClientId, PlayerJumpEvent},
    player::JumpEvent,
    GameState, InputState,
};

use crate::{camera, connect_server, input, main_menu, ui, AppState, Settings};

#[derive(Debug, Default, Resource)]
pub struct ClientState {
    host: Option<String>,
}

impl ClientState {
    #[inline]
    pub fn new_remote(host: impl Into<String>) -> Self {
        Self {
            host: Some(host.into()),
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
        ))
        .init_resource::<Settings>()
        .init_resource::<ClientState>()
        .add_systems(OnEnter(AppState::InGame), enter)
        .add_systems(
            PostUpdate,
            (send_input_update, send_jump_event)
                .before(ClientSet::Send)
                .run_if(in_state(AppState::InGame))
                .run_if(client_connected),
        )
        .add_systems(OnExit(AppState::InGame), exit);
    }
}

fn enter(mut game_state: ResMut<NextState<GameState>>) {
    info!("enter client game ...");

    game_state.set(GameState::LoadAssets);
}

fn exit(mut commands: Commands) {
    info!("exit client game ...");

    commands.remove_resource::<PlayerClientId>();
    commands.remove_resource::<ClientState>();
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
}

pub fn on_connected_server(
    client: &ClientState,
    client_id: ClientId,
    app_state: &mut NextState<AppState>,
) {
    if client.is_local() {
        info!("running local");
    } else {
        info!("connected to {:?} as {:?}", client.host(), client_id);
    }

    app_state.set(AppState::InGame);
}

fn send_input_update(
    mut input: ResMut<InputState>,
    mut evw_input_update: EventWriter<InputUpdateEvent>,
) {
    evw_input_update.send(InputUpdateEvent(*input));

    input.look = Vec2::default();
    input.r#move = Vec2::default();
}

fn send_jump_event(
    mut evr_jump: EventReader<JumpEvent>,
    mut evw_jump: EventWriter<PlayerJumpEvent>,
) {
    // TODO: only send if the player is grounded
    if !evr_jump.is_empty() {
        evw_jump.send(PlayerJumpEvent);
        evr_jump.clear();
    }
}
