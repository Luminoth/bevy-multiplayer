use bevy::prelude::*;
use bevy_mod_websocket::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::renet::{transport::NetcodeClientTransport, RenetClient};

use common::user::UserId;
use game_common::{
    network::{ConnectEvent, InputUpdateEvent, PlayerClientId, PlayerJumpEvent},
    player, GameState, InputState,
};

use crate::{
    camera, connect_server, game, game_menu, input, main_menu, notifs, options::Options, ui,
    AppState, Settings,
};

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
            game_menu::GameMenuPlugin,
            connect_server::ConnectServerPlugin,
            camera::FpsCameraPlugin,
            input::InputPlugin,
            ui::UiPlugin,
            game::GamePlugin,
        ))
        .init_resource::<Settings>()
        .init_resource::<ClientState>()
        .add_systems(Startup, setup)
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

fn setup(options: Res<Options>, mut ws_client: WebSocketClient) {
    info!("starting client {}", options.user_id);

    notifs::subscribe(&mut ws_client, options.user_id);
}

fn enter(mut game_state: ResMut<NextState<GameState>>) {
    info!("enter client game ...");

    game_state.set(GameState::LoadAssets);
}

fn exit(mut commands: Commands) {
    info!("exit client game ...");

    commands.init_resource::<ClientState>();

    commands.remove_resource::<PlayerClientId>();
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
}

pub fn on_connected_server(
    client: &ClientState,
    client_id: PlayerClientId,
    user_id: UserId,
    evw_connect: &mut EventWriter<ConnectEvent>,
    app_state: &mut NextState<AppState>,
) {
    if client_id.is_local() {
        info!("running local");
    } else {
        info!(
            "connected to {:?} as {:?}",
            client.host(),
            client_id.get_client_id()
        );
    }

    evw_connect.send(ConnectEvent(user_id));

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
    mut evr_jump: EventReader<input::JumpPressedEvent>,
    mut evw_jump: EventWriter<PlayerJumpEvent>,
    player_query: Query<&player::PlayerPhysics, With<player::LocalPlayer>>,
) {
    if !evr_jump.is_empty() {
        let player_physics = player_query.single();
        if player_physics.is_grounded() {
            evw_jump.send(PlayerJumpEvent);
            evr_jump.clear();
        }
    }
}
