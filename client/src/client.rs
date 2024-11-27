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
    api, camera, connect_server, game, game_menu, input, main_menu, options::Options, ui, AppState,
    Settings,
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

    api::subscribe(&mut ws_client, options.user_id)
        .on_success(|trigger: Trigger<WebSocketConnectSuccessEvent>| {
            let evt = trigger.event();
            info!("subscribe success: {:?}", evt);
        })
        .on_error(|trigger: Trigger<WebSocketErrorEvent>| {
            let evt = trigger.event();
            // TODO: temp panic until we have retry
            //warn!("notifs error: {:?}", evt);
            panic!("notifs error: {:?}", evt);
        })
        .on_message(|trigger: Trigger<WebSocketMessageEvent>| {
            let evt = trigger.event();

            match &evt.message {
                Message::Text(value) => {
                    info!("received notif from {}: {:?}", evt.uri, value);

                    // TODO: error handling
                    //let notif = serde_json::from_str::<notifs::Notification>(value).unwrap();
                    //match notif.r#type {}
                }
                _ => {
                    warn!("unexpected notif from {}: {:?}", evt.uri, evt.message);
                }
            }
        })
        .on_disconnect(|trigger: Trigger<WebSocketDisconnectEvent>| {
            let evt = trigger.event();
            // TODO: temp panic until we have reconnect
            //warn!("notifs disconnect: {:?}", evt);
            panic!("notifs disconnect: {:?}", evt);
        });
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
