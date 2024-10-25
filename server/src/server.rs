use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_reqwest::*;
use bevy_replicon_renet::renet::transport::{
    NetcodeServerTransport, ServerAuthentication, ServerConfig,
};
use bevy_replicon_renet::renet::ServerEvent;
use uuid::Uuid;

use game::{GameState, PROTOCOL_ID};

use crate::{api, AppState};

#[derive(Debug, Resource)]
pub struct GameServerInfo {
    pub server_id: Uuid,
}

impl GameServerInfo {
    pub fn new() -> Self {
        Self {
            server_id: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Resource)]
pub struct GameSessionInfo {
    pub session_id: Uuid,
    pub player_session_ids: Vec<Uuid>,
    pub pending_player_ids: Vec<String>,
}

fn heartbeat(
    client: &mut BevyReqwest,
    server_info: &GameServerInfo,
    session_info: Option<&GameSessionInfo>,
) {
    api::heartbeat(client, server_info, session_info).on_error(
        |trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            error!("heartbeat error: {:?}", e);
        },
    );
}

#[derive(Debug)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy_replicon_renet::renet::RenetServer::new(
            bevy_replicon_renet::renet::ConnectionConfig::default(),
        ))
        // rapier makes use of Mesh assets
        // and this is missing without rendering
        .init_asset::<Mesh>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                wait_for_placement.run_if(in_state(AppState::WaitForPlacement)),
                init_network.run_if(in_state(AppState::InitServer)),
                handle_network_events.run_if(in_state(GameState::InGame)),
                heartbeat_monitor.run_if(on_timer(Duration::from_secs(30))),
            ),
        )
        .add_systems(OnEnter(AppState::InGame), enter)
        .add_systems(OnExit(AppState::InGame), exit);
    }
}

fn setup(mut commands: Commands, mut client: BevyReqwest) {
    let server_info = GameServerInfo::new();
    info!("starting server {}", server_info.server_id);

    heartbeat(&mut client, &server_info, None);

    commands.insert_resource(server_info);
}

fn enter(mut game_state: ResMut<NextState<GameState>>) {
    info!("enter game ...");

    game_state.set(GameState::LoadAssets);
}

fn exit(mut commands: Commands) {
    info!("exit game ...");

    commands.remove_resource::<NetcodeServerTransport>();
}

fn heartbeat_monitor(
    mut client: BevyReqwest,
    server_info: Res<GameServerInfo>,
    session_info: Option<Res<GameSessionInfo>>,
) {
    let session_info = session_info.as_deref();
    heartbeat(&mut client, &server_info, session_info);
}

fn wait_for_placement(
    mut commands: Commands,
    mut client: BevyReqwest,
    server_info: Res<GameServerInfo>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    warn!("faking placement!");

    let session_info = GameSessionInfo {
        session_id: Uuid::new_v4(),
        player_session_ids: vec![],
        pending_player_ids: vec!["test_player".into()],
    };
    info!("starting session {}", session_info.session_id);

    heartbeat(&mut client, &server_info, Some(&session_info));

    commands.insert_resource(session_info);

    app_state.set(AppState::InitServer);
}

fn init_network(mut commands: Commands, mut app_state: ResMut<NextState<AppState>>) {
    info!("init network ...");

    // TODO: this should bind a specific address
    let server_addr = "0.0.0.0:5576".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 3,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    info!("listening at {} ...", server_addr);

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    commands.insert_resource(transport);

    app_state.set(AppState::InGame);
}

fn handle_network_events(mut evt_server: EventReader<ServerEvent>) {
    for evt in evt_server.read() {
        match evt {
            ServerEvent::ClientConnected { client_id } => {
                info!("client {} connected", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client {} disconnected: {}", client_id, reason);
            }
        }
    }
}
