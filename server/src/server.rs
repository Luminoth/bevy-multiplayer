use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_reqwest::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ConnectionConfig, RenetServer,
    },
    RenetChannelsExt,
};
use bevy_tokio_tasks::TokioTasksRuntime;
use uuid::Uuid;

use common::gameserver::GameServerState;
use game_common::{
    network::{InputUpdateEvent, PlayerJumpEvent},
    player,
    spawn::SpawnPoint,
    GameAssetState, GameState, PROTOCOL_ID,
};

use crate::{
    api, game, options::Options, orchestration::Orchestration, placement, tasks, AppState,
};

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
        app.add_plugins((placement::PlacementPlugin, game::GamePlugin))
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                (handle_input_update, handle_jump_event)
                    .after(ServerSet::Receive)
                    .run_if(in_state(AppState::InGame))
                    .run_if(server_running),
            )
            .add_systems(
                Update,
                (
                    handle_network_events.run_if(in_state(GameState::InGame)),
                    heartbeat_monitor.run_if(on_timer(Duration::from_secs(30))),
                ),
            )
            .add_systems(OnEnter(AppState::InitServer), init_server)
            .add_systems(OnEnter(AppState::InGame), enter)
            .add_systems(OnExit(AppState::InGame), exit)
            .add_systems(OnEnter(AppState::Shutdown), shutdown);
    }
}

fn setup(
    mut commands: Commands,
    options: Res<Options>,
    mut client: BevyReqwest,
    runtime: Res<TokioTasksRuntime>,
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
        &runtime,
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

fn shutdown(orchestration: Res<Orchestration>, runtime: Res<TokioTasksRuntime>) {
    let orchestration = orchestration.clone();
    orchestration.stop_watcher();

    tasks::spawn_task(
        &runtime,
        move || async move { orchestration.shutdown().await },
        |ctx, _output| {
            ctx.world.send_event(AppExit::Success);
        },
        |ctx, err| {
            error!("orchestration shutdown error: {}", err);
            ctx.world.send_event(AppExit::from_code(1));
        },
    );
}

fn enter(
    mut client: BevyReqwest,
    server_info: Res<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
    state: Res<State<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    info!("enter server game ...");

    heartbeat(
        &mut client,
        server_info.server_id,
        (**state).into(),
        Some(&session_info),
    );

    game_state.set(GameState::LoadAssets);
}

fn exit(mut commands: Commands) {
    info!("exit server game ...");

    commands.remove_resource::<GameSessionInfo>();
    commands.remove_resource::<RenetServer>();
    commands.remove_resource::<NetcodeServerTransport>();
}

fn heartbeat_monitor(
    mut client: BevyReqwest,
    orchestration: Res<Orchestration>,
    server_info: Res<GameServerInfo>,
    state: Res<State<AppState>>,
    session_info: Option<Res<GameSessionInfo>>,
    runtime: Res<TokioTasksRuntime>,
) {
    let session_info = session_info.as_deref();
    heartbeat(
        &mut client,
        server_info.server_id,
        (**state).into(),
        session_info,
    );

    if state.is_ready() {
        let orchestration = orchestration.clone();
        tasks::spawn_task(
            &runtime,
            move || async move { orchestration.health_check().await },
            |_ctx, _output| {},
            |_ctx, err| {
                error!("failed orchestration health check: {}", err);
            },
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn init_server(
    mut commands: Commands,
    mut client: BevyReqwest,
    options: Res<Options>,
    channels: Res<RepliconChannels>,
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

    let server_addr = options.address().parse().unwrap();
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

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config: channels.get_server_configs(),
        client_channels_config: channels.get_client_configs(),
        ..Default::default()
    });
    commands.insert_resource(server);

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    commands.insert_resource(transport);

    app_state.set(AppState::InGame);
}

// TODO: we shouldn't allow connections until we've loaded assets
// (otherwise spawning the player will probably fail)
fn handle_network_events(
    mut commands: Commands,
    assets: Res<GameAssetState>,
    spawnpoints: Query<&GlobalTransform, With<SpawnPoint>>,
    players: Query<(Entity, &player::Player)>,
    mut evr_server: EventReader<ServerEvent>,
) {
    for evt in evr_server.read() {
        match evt {
            ServerEvent::ClientConnected { client_id } => {
                info!("client {:?} connected", client_id);

                let spawnpoint = spawnpoints.iter().next().unwrap();
                player::spawn_player(&mut commands, *client_id, spawnpoint.translation(), &assets);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client {:?} disconnected: {}", client_id, reason);

                for (entity, player) in players.iter() {
                    if player.client_id() == *client_id {
                        player::despawn_player(&mut commands, entity);
                    }
                }
            }
        }
    }
}

fn handle_input_update(
    mut evr_input_update: EventReader<FromClient<InputUpdateEvent>>,
    mut player_query: Query<(&mut player::LastInput, &player::Player)>,
) {
    for FromClient { client_id, event } in evr_input_update.read() {
        for (mut last_input, player) in &mut player_query {
            if player.client_id() == *client_id {
                last_input.input_state = event.0;
            }
        }
    }
}

fn handle_jump_event(
    mut evr_jump: EventReader<FromClient<PlayerJumpEvent>>,
    mut player_query: Query<(&mut player::LastInput, &player::Player)>,
) {
    for FromClient {
        client_id,
        event: _,
    } in evr_jump.read()
    {
        for (mut last_input, player) in &mut player_query {
            if player.client_id() == *client_id {
                last_input.jump = true;
            }
        }
    }
}
