use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_reqwest::*;
use bevy_replicon_renet::renet::transport::{
    NetcodeServerTransport, ServerAuthentication, ServerConfig,
};
use bevy_replicon_renet::renet::ServerEvent;
use bevy_tokio_tasks::TokioTasksRuntime;
use uuid::Uuid;

use common::gameserver::GameServerState;
use game::{GameState, PROTOCOL_ID};

use crate::{api, options::Options, orchestration::Orchestration, placement, tasks, AppState};

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

pub fn heartbeat(
    client: &mut BevyReqwest,
    server_id: Uuid,
    state: GameServerState,
    // TODO: connection info
    session_info: Option<&GameSessionInfo>,
) {
    api::heartbeat(client, server_id, state, session_info).on_error(
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
        app.add_plugins(placement::PlacementPlugin)
            .insert_resource(bevy_replicon_renet::renet::RenetServer::new(
                bevy_replicon_renet::renet::ConnectionConfig::default(),
            ))
            // rapier makes use of Mesh assets
            // and this is missing without rendering
            .init_asset::<Mesh>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_network_events.run_if(in_state(GameState::InGame)),
                    heartbeat_monitor.run_if(on_timer(Duration::from_secs(30))),
                ),
            )
            .add_systems(OnEnter(AppState::InitServer), init_server)
            .add_systems(OnEnter(AppState::InGame), enter)
            .add_systems(OnExit(AppState::InGame), exit);
    }
}

fn setup(
    mut commands: Commands,
    options: Res<Options>,
    mut client: BevyReqwest,
    mut runtime: ResMut<TokioTasksRuntime>,
) {
    let server_info = GameServerInfo::new();
    info!("starting server {}", server_info.server_id);

    // let the backend know we're starting up
    heartbeat(
        &mut client,
        server_info.server_id,
        AppState::Startup.into(),
        None,
    );

    commands.insert_resource(server_info);

    let orchestration_type = options.orchestration;
    tasks::spawn_task(
        &mut runtime,
        move || async move { Orchestration::new(orchestration_type).await },
        |ctx, output| {
            ctx.world.insert_resource(output);

            let mut app_state = ctx.world.resource_mut::<NextState<AppState>>();
            app_state.set(AppState::WaitForPlacement);
        },
        |_ctx, err| {
            panic!("failed to init orchestration backend: {}", err);
        },
    );
}

fn enter(
    mut game_state: ResMut<NextState<GameState>>,
    mut client: BevyReqwest,
    server_info: Res<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
    state: Res<State<AppState>>,
) {
    info!("enter game ...");

    heartbeat(
        &mut client,
        server_info.server_id,
        (**state).into(),
        Some(&session_info),
    );

    game_state.set(GameState::LoadAssets);
}

fn exit(mut commands: Commands) {
    info!("exit game ...");

    commands.remove_resource::<GameSessionInfo>();
    commands.remove_resource::<NetcodeServerTransport>();
}

fn heartbeat_monitor(
    mut client: BevyReqwest,
    server_info: Res<GameServerInfo>,
    state: Res<State<AppState>>,
    session_info: Option<Res<GameSessionInfo>>,
) {
    let session_info = session_info.as_deref();
    heartbeat(
        &mut client,
        server_info.server_id,
        (**state).into(),
        session_info,
    );
}

fn init_server(
    mut commands: Commands,
    mut client: BevyReqwest,
    options: Res<Options>,
    server_info: Res<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
    current_state: Res<State<AppState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    info!("init network ...");

    // let the backend know we're initializing the game
    heartbeat(
        &mut client,
        server_info.server_id,
        (**current_state).into(),
        Some(&session_info),
    );

    // TODO: this should bind a specific address
    let server_addr = format!("0.0.0.0:{}", options.port).parse().unwrap();
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
