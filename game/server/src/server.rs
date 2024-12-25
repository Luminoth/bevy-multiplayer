use std::collections::{BTreeSet, HashMap, HashSet};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

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
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use uuid::Uuid;

use common::{
    gameserver::{GameServerOrchestration, GameServerState},
    user::UserId,
};
use game_common::{
    network::{ConnectEvent, InputUpdateEvent, PlayerJumpEvent},
    player,
    spawn::SpawnPoint,
    GameAssetState, GameState, PROTOCOL_ID,
};

use crate::{
    api, game, notifs, options::Options, orchestration::Orchestration, placement, tasks, AppState,
};

#[derive(Debug, Clone, Default)]
pub struct ConnectionInfo {
    pub v4addrs: BTreeSet<String>,
    pub v6addrs: BTreeSet<String>,
    pub port: u16,
}

impl ConnectionInfo {
    pub fn update(&mut self, addr: SocketAddr) {
        let ip = addr.ip();
        if ip.is_unspecified() {
            self.v4addrs.clear();
            self.v6addrs.clear();

            let ifaces = NetworkInterface::show().unwrap();
            for iface in ifaces {
                // hack for now, I honestly don't know how to ignore this
                if iface.name.contains("docker") {
                    continue;
                }

                // also a hack, I don't know how to ignore bridge interfaces
                if iface.name.starts_with("br-") {
                    continue;
                }

                for ip in iface.addr {
                    let ip = ip.ip();
                    // TODO: copy paste
                    match ip {
                        IpAddr::V4(ip) => {
                            if !ip.is_loopback() && !ip.is_link_local() {
                                self.v4addrs.insert(ip.to_string());
                            }
                        }
                        IpAddr::V6(ip) => {
                            if !ip.is_loopback() {
                                self.v6addrs.insert(ip.to_string());
                            }
                        }
                    }
                }
            }
        } else {
            // TODO: copy paste
            match ip {
                IpAddr::V4(ip) => {
                    if !ip.is_loopback() && !ip.is_link_local() {
                        self.v4addrs.insert(ip.to_string());
                    }
                }
                IpAddr::V6(ip) => {
                    if !ip.is_loopback() {
                        self.v6addrs.insert(ip.to_string());
                    }
                }
            }
        }
        self.port = addr.port();

        info!("updated connection info: {:?}", self);
    }
}

#[derive(Debug, Resource)]
pub struct GameServerInfo {
    pub server_id: Uuid,
    pub connection_info: ConnectionInfo,
}

impl GameServerInfo {
    pub fn new() -> Self {
        Self {
            server_id: Uuid::new_v4(),
            connection_info: ConnectionInfo::default(),
        }
    }
}

#[derive(Debug, Resource)]
pub struct GameSessionInfo {
    pub session_id: Uuid,
    pub max_players: u16,

    pub active_player_ids: HashSet<UserId>,
    pub pending_player_ids: HashSet<UserId>,

    clients: HashMap<ClientId, UserId>,
}

impl GameSessionInfo {
    pub fn new(
        session_id: Uuid,
        settings: &internal::GameSettings,
        pending_player_ids: Vec<UserId>,
    ) -> Self {
        Self {
            session_id,
            max_players: settings.max_players,
            active_player_ids: HashSet::with_capacity(settings.max_players as usize),
            pending_player_ids: HashSet::from_iter(pending_player_ids),
            clients: HashMap::with_capacity(settings.max_players as usize),
        }
    }

    #[inline]
    pub fn player_count(&self) -> usize {
        self.active_player_ids.len() + self.pending_player_ids.len()
    }

    fn client_connected(&mut self, user_id: UserId, client_id: ClientId) -> bool {
        if self.pending_player_ids.contains(&user_id) {
            self.pending_player_ids.remove(&user_id);
            self.active_player_ids.insert(user_id);
            self.clients.insert(client_id, user_id);

            true
        } else {
            false
        }
    }

    fn client_disconnected(&mut self, client_id: &ClientId) {
        if let Some(user_id) = self.clients.remove(client_id) {
            self.active_player_ids.remove(&user_id);
            self.pending_player_ids.remove(&user_id);
        }
    }
}

// TODO: heartbeat off an event instead
// so we don't have to pass all this garbage into everything
pub fn heartbeat(
    client: &mut BevyReqwest,
    server_id: Uuid,
    connection_info: ConnectionInfo,
    state: GameServerState,
    orchestration: GameServerOrchestration,
    session_info: Option<&GameSessionInfo>,
) {
    api::heartbeat(
        client,
        server_id,
        connection_info,
        state,
        orchestration,
        session_info,
    )
    .unwrap();
}

#[derive(Debug)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((placement::PlacementPlugin, game::GamePlugin))
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                (handle_connect, handle_input_update, handle_jump_event)
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
    mut ws_client: WebSocketClient,
    runtime: Res<TokioTasksRuntime>,
) {
    let server_info = GameServerInfo::new();
    info!("starting server {}", server_info.server_id);

    // let the backend know we're starting up
    heartbeat(
        &mut client,
        server_info.server_id,
        server_info.connection_info.clone(),
        AppState::Startup.into(),
        GameServerOrchestration::Local,
        None,
    );

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
    mut client: BevyReqwest,
    orchestration: Res<Orchestration>,
    server_info: Res<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
    state: Res<State<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    info!("enter server game ...");

    heartbeat(
        &mut client,
        server_info.server_id,
        server_info.connection_info.clone(),
        (**state).into(),
        orchestration.as_api_type(),
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
        server_info.connection_info.clone(),
        (**state).into(),
        orchestration.as_api_type(),
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
    orchestration: Res<Orchestration>,
    mut server_info: ResMut<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
    current_state: Res<State<AppState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    info!("init network ...");

    // let the backend know we're initializing the game
    heartbeat(
        &mut client,
        server_info.server_id,
        server_info.connection_info.clone(),
        (**current_state).into(),
        orchestration.as_api_type(),
        None,
    );

    let server_addr = options.address().parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
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
    mut client: BevyReqwest,
    app_state: Res<State<AppState>>,
    server_info: Res<GameServerInfo>,
    mut session_info: ResMut<GameSessionInfo>,
    orchestration: Res<Orchestration>,
    players: Query<(Entity, &player::Player)>,
    mut evr_server: EventReader<ServerEvent>,
) {
    for evt in evr_server.read() {
        match evt {
            ServerEvent::ClientConnected { client_id } => {
                info!("client {:?} connected", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client {:?} disconnected: {}", client_id, reason);

                for (entity, player) in players.iter() {
                    if player.client_id() == *client_id {
                        player::despawn_player(&mut commands, entity);
                    }
                }

                session_info.client_disconnected(client_id);

                heartbeat(
                    &mut client,
                    server_info.server_id,
                    server_info.connection_info.clone(),
                    (**app_state).into(),
                    orchestration.as_api_type(),
                    Some(&session_info),
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_connect(
    mut commands: Commands,
    mut client: BevyReqwest,
    mut evr_connect: EventReader<FromClient<ConnectEvent>>,
    app_state: Res<State<AppState>>,
    assets: Option<Res<GameAssetState>>,
    mut server: ResMut<RenetServer>,
    server_info: Res<GameServerInfo>,
    mut session_info: ResMut<GameSessionInfo>,
    orchestration: Res<Orchestration>,
    spawnpoints: Query<&GlobalTransform, With<SpawnPoint>>,
) {
    for FromClient { client_id, event } in evr_connect.read() {
        let user_id = event.0;
        info!("player {} connected", user_id);

        if !session_info.client_connected(user_id, *client_id) {
            warn!("player {} not expected", user_id);
            server.disconnect(client_id.get());
            continue;
        }

        heartbeat(
            &mut client,
            server_info.server_id,
            server_info.connection_info.clone(),
            (**app_state).into(),
            orchestration.as_api_type(),
            Some(&session_info),
        );

        let spawnpoint = spawnpoints.iter().next().unwrap();
        player::spawn_player(
            &mut commands,
            *client_id,
            spawnpoint.translation(),
            assets.as_ref().unwrap(),
        );
    }
}

fn handle_input_update(
    mut evr_input_update: EventReader<FromClient<InputUpdateEvent>>,
    session_info: Res<GameSessionInfo>,
    mut server: ResMut<RenetServer>,
    mut player_query: Query<(&mut player::LastInput, &player::Player)>,
) {
    for FromClient { client_id, event } in evr_input_update.read() {
        if !session_info.clients.contains_key(client_id) {
            warn!("client {:?} not in session", client_id);
            server.disconnect(client_id.get());
            continue;
        }

        for (mut last_input, player) in &mut player_query {
            if player.client_id() == *client_id {
                last_input.input_state = event.0;
            }
        }
    }
}

fn handle_jump_event(
    mut evr_jump: EventReader<FromClient<PlayerJumpEvent>>,
    session_info: Res<GameSessionInfo>,
    mut server: ResMut<RenetServer>,
    mut player_query: Query<(&mut player::LastInput, &player::Player)>,
) {
    for FromClient {
        client_id,
        event: _,
    } in evr_jump.read()
    {
        if !session_info.clients.contains_key(client_id) {
            warn!("client {:?} not in session", client_id);
            server.disconnect(client_id.get());
            continue;
        }

        for (mut last_input, player) in &mut player_query {
            if player.client_id() == *client_id {
                last_input.jump = true;
            }
        }
    }
}
