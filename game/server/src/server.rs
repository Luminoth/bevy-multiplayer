use std::net::UdpSocket;
use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_reqwest::*;
use bevy_mod_websocket::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, RenetServer},
    RenetChannelsExt,
};
use bevy_tokio_tasks::TokioTasksRuntime;

use game_common::{
    network::{ConnectEvent, InputUpdateEvent, PlayerJumpEvent},
    player,
    server::{ActivePlayer, GameServerInfo, GameSessionInfo, PendingPlayer},
    spawn::SpawnPoint,
    utils::current_timestamp,
    GameAssetState, GameState, PROTOCOL_ID,
};

use crate::{
    api, game, notifs, options::Options, orchestration::Orchestration, placement, tasks, AppState,
};

const HEARTBEAT_FREQUENCY: Duration = Duration::from_secs(5);

#[derive(Debug, Default, Event)]
pub struct HeartbeatEvent;

#[derive(Debug)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((placement::PlacementPlugin, game::GamePlugin))
            .add_event::<HeartbeatEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                (handle_connect, validate_input_update, validate_jump_event)
                    .after(ServerSet::Receive)
                    .run_if(in_state(AppState::InGame))
                    .run_if(server_running),
            )
            .add_systems(
                Update,
                (
                    handle_network_events.run_if(in_state(GameState::InGame)),
                    handle_timeouts.run_if(in_state(GameState::InGame)),
                    heartbeat_monitor.run_if(on_timer(HEARTBEAT_FREQUENCY)),
                    handle_heartbeat_events,
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
    mut ws_client: WebSocketClient,
    runtime: Res<TokioTasksRuntime>,
    mut evw_heartbeat: EventWriter<HeartbeatEvent>,
) {
    let server_info = GameServerInfo::new();
    info!("starting server {}", server_info.server_id);

    // let the backend know we're starting up
    evw_heartbeat.send_default();

    notifs::subscribe(&mut ws_client, server_info.server_id);

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
    mut game_state: ResMut<NextState<GameState>>,
    mut evw_heartbeat: EventWriter<HeartbeatEvent>,
) {
    info!("entering server app game ...");

    evw_heartbeat.send_default();

    game_state.set(GameState::LoadAssets);
}

fn exit(mut commands: Commands) {
    info!("exiting server app game ...");

    commands.remove_resource::<GameSessionInfo>();
    commands.remove_resource::<RenetServer>();
    commands.remove_resource::<NetcodeServerTransport>();
}

fn heartbeat_monitor(
    orchestration: Res<Orchestration>,
    state: Res<State<AppState>>,
    runtime: Res<TokioTasksRuntime>,
    mut evw_heartbeat: EventWriter<HeartbeatEvent>,
) {
    evw_heartbeat.send_default();

    // send orchestration health check
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
fn handle_heartbeat_events(
    mut client: BevyReqwest,
    orchestration: Option<Res<Orchestration>>,
    server_info: Res<GameServerInfo>,
    session_info: Option<Res<GameSessionInfo>>,
    state: Res<State<AppState>>,
    pending_players: Query<&PendingPlayer>,
    active_players: Query<&ActivePlayer>,
    mut evr_heartbeat: EventReader<HeartbeatEvent>,
) {
    if let Some(orchestration) = orchestration {
        if !evr_heartbeat.is_empty() {
            api::heartbeat(
                &mut client,
                server_info.server_id,
                server_info.connection_info.clone(),
                (**state).into(),
                orchestration.as_api_type(),
                session_info.as_deref(),
                pending_players.iter(),
                active_players.iter(),
            )
            .unwrap();
        }
    }

    evr_heartbeat.clear();
}

#[allow(clippy::too_many_arguments)]
fn init_server(
    mut commands: Commands,
    options: Res<Options>,
    channels: Res<RepliconChannels>,
    mut server_info: ResMut<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
    mut app_state: ResMut<NextState<AppState>>,
    mut evw_heartbeat: EventWriter<HeartbeatEvent>,
) {
    info!("init network ...");

    // let the backend know we're initializing the game
    evw_heartbeat.send_default();

    let server_addr = options.address().parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let current_time = current_timestamp();
    let server_config = ServerConfig {
        current_time,
        max_clients: session_info.max_players as usize,
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

    server_info.connection_info.update(server_addr);

    app_state.set(AppState::InGame);
}

// TODO: we shouldn't allow connections until we've loaded assets
// (otherwise spawning the player will probably fail)
#[allow(clippy::too_many_arguments)]
fn handle_network_events(
    mut commands: Commands,
    mut session_info: ResMut<GameSessionInfo>,
    pending_players: Query<(Entity, &PendingPlayer)>,
    active_players: Query<(Entity, &ActivePlayer)>,
    players: Query<(Entity, &player::Player)>,
    mut evr_server: EventReader<ServerEvent>,
    mut evw_heartbeat: EventWriter<HeartbeatEvent>,
) {
    for evt in evr_server.read() {
        match evt {
            ServerEvent::ClientConnected { client_id } => {
                info!("client {:?} connected", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client {:?} disconnected: {}", client_id, reason);

                for (entity, player) in &players {
                    if player.client_id == *client_id {
                        player::despawn_player(&mut commands, entity, player.user_id);
                    }
                }

                session_info.client_disconnected(
                    &mut commands,
                    client_id,
                    pending_players.iter(),
                    active_players.iter(),
                );

                evw_heartbeat.send_default();
            }
        }
    }
}

fn handle_timeouts(
    mut commands: Commands,
    time: Res<Time>,
    orchestration: Res<Orchestration>,
    mut session_info: ResMut<GameSessionInfo>,
    mut pending_players: Query<(Entity, &mut PendingPlayer)>,
    mut exit: EventWriter<AppExit>,
) {
    for (entity, mut pending_player) in &mut pending_players {
        if pending_player.is_timeout(time.delta()) {
            session_info.pending_player_timeout(&mut commands, entity, pending_player.user_id);
        }
    }

    if orchestration.shutdown_empty() && session_info.update_shutdown_timer(time.delta()) {
        info!("session timeout, exiting");
        exit.send(AppExit::Success);
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_connect(
    mut commands: Commands,
    mut evr_connect: EventReader<FromClient<ConnectEvent>>,
    assets: Option<Res<GameAssetState>>,
    mut server: ResMut<RenetServer>,
    mut session_info: ResMut<GameSessionInfo>,
    pending_players: Query<(Entity, &PendingPlayer)>,
    spawnpoints: Query<&GlobalTransform, With<SpawnPoint>>,
    mut evw_heartbeat: EventWriter<HeartbeatEvent>,
) {
    for FromClient { client_id, event } in evr_connect.read() {
        let user_id = event.0;
        info!("player {} connected", user_id);

        if !session_info.client_connected(
            &mut commands,
            user_id,
            *client_id,
            pending_players.iter(),
        ) {
            warn!("player {} not expected", user_id);
            server.disconnect(client_id.get());
            continue;
        }

        evw_heartbeat.send_default();

        let spawnpoint = spawnpoints.iter().next().unwrap();
        player::spawn_player(
            &mut commands,
            user_id,
            *client_id,
            spawnpoint.translation(),
            assets.as_ref().unwrap(),
        );
    }
}

fn validate_input_update(
    mut evr_input_update: EventReader<FromClient<InputUpdateEvent>>,
    session_info: Res<GameSessionInfo>,
    mut server: ResMut<RenetServer>,
) {
    for FromClient {
        client_id,
        event: _,
    } in evr_input_update.read()
    {
        if !session_info.has_client(client_id) {
            warn!("client {:?} not in session", client_id);
            server.disconnect(client_id.get());
            continue;
        }

        // shared game handles the event
    }
}

fn validate_jump_event(
    mut evr_jump: EventReader<FromClient<PlayerJumpEvent>>,
    session_info: Res<GameSessionInfo>,
    mut server: ResMut<RenetServer>,
) {
    for FromClient {
        client_id,
        event: _,
    } in evr_jump.read()
    {
        if !session_info.has_client(client_id) {
            warn!("client {:?} not in session", client_id);
            server.disconnect(client_id.get());
            continue;
        }

        // shared game handles the event
    }
}
