use std::collections::HashMap;
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
use uuid::Uuid;

use common::user::UserId;
use game_common::{
    network::{ConnectEvent, ConnectionInfo, InputUpdateEvent, PlayerJumpEvent},
    player,
    spawn::SpawnPoint,
    utils::current_timestamp,
    GameAssetState, GameState, PROTOCOL_ID,
};

use crate::{
    api, game, notifs, options::Options, orchestration::Orchestration, placement, tasks, AppState,
};

const HEARTBEAT_FREQUENCY: Duration = Duration::from_secs(5);
const PENDING_PLAYER_TIMEOUT: Duration = Duration::from_secs(10);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 10);

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

#[derive(Debug, Component)]
pub struct PendingPlayer {
    pub user_id: UserId,
    timer: Timer,
}

impl PendingPlayer {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            timer: Timer::new(PENDING_PLAYER_TIMEOUT, TimerMode::Once),
        }
    }

    pub fn is_timeout(&mut self, delta: Duration) -> bool {
        self.timer.tick(delta);
        self.timer.finished()
    }
}

#[derive(Debug, Component)]
pub struct ActivePlayer {
    pub user_id: UserId,
}

impl ActivePlayer {
    fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

#[derive(Debug, Resource)]
pub struct GameSessionInfo {
    pub session_id: Uuid,
    pub max_players: u16,

    pending_player_count: usize,
    active_player_count: usize,

    clients: HashMap<ClientId, UserId>,

    shutdown_timer: Timer,
}

impl GameSessionInfo {
    pub fn new(
        commands: &mut Commands,
        session_id: Uuid,
        settings: &internal::GameSettings,
        pending_player_ids: impl AsRef<[UserId]>,
    ) -> Self {
        let mut this = Self {
            session_id,
            max_players: settings.max_players,
            pending_player_count: 0,
            active_player_count: 0,
            clients: HashMap::with_capacity(settings.max_players as usize),
            shutdown_timer: Timer::new(SHUTDOWN_TIMEOUT, TimerMode::Once),
        };
        this.shutdown_timer.pause();

        for pending_player_id in pending_player_ids.as_ref().iter() {
            this.reserve_player(commands, *pending_player_id);
        }

        this
    }

    #[inline]
    pub fn player_count(&self) -> usize {
        self.pending_player_count + self.active_player_count
    }

    pub fn reserve_player(&mut self, commands: &mut Commands, pending_player_id: UserId) {
        if self.player_count() + 1 > self.max_players as usize {
            warn!(
                "not reserving player slot for {}, max players {} exceeded!",
                pending_player_id, self.max_players
            );
            return;
        }

        info!("reserving player slot {}", pending_player_id);

        commands.spawn(PendingPlayer::new(pending_player_id));
        self.pending_player_count += 1;

        self.shutdown_timer.pause();
    }

    fn pending_player_timeout(
        &mut self,
        commands: &mut Commands,
        pending_player: Entity,
        pending_player_id: UserId,
    ) {
        info!("pending player {} timeout", pending_player_id);

        commands.entity(pending_player).despawn_recursive();
        self.pending_player_count -= 1;

        if self.player_count() == 0 {
            self.shutdown_timer.reset();
            self.shutdown_timer.unpause();
        }
    }

    fn client_connected<'a>(
        &mut self,
        commands: &mut Commands,
        user_id: UserId,
        client_id: ClientId,
        mut pending_players: impl Iterator<Item = (Entity, &'a PendingPlayer)>,
    ) -> bool {
        let pending_player = pending_players.find_map(|v| {
            if v.1.user_id == user_id {
                Some(v.0)
            } else {
                None
            }
        });
        if let Some(pending_player) = pending_player {
            info!("activating player slot {} for {:?}", user_id, client_id);

            commands.entity(pending_player).despawn_recursive();
            self.pending_player_count -= 1;

            commands.spawn(ActivePlayer::new(user_id));
            self.active_player_count += 1;

            self.clients.insert(client_id, user_id);

            self.shutdown_timer.pause();

            true
        } else {
            false
        }
    }

    fn client_disconnected<'a>(
        &mut self,
        commands: &mut Commands,
        client_id: &ClientId,
        mut pending_players: impl Iterator<Item = (Entity, &'a PendingPlayer)>,
        mut active_players: impl Iterator<Item = (Entity, &'a ActivePlayer)>,
    ) {
        if let Some(user_id) = self.clients.remove(client_id) {
            if let Some(pending_player) = pending_players.find_map(|v| {
                if v.1.user_id == user_id {
                    Some(v.0)
                } else {
                    None
                }
            }) {
                info!("pending player {} disconnected ?", user_id);

                commands.entity(pending_player).despawn_recursive();
                self.pending_player_count -= 1;
            }

            let active_player = active_players.find_map(|v| {
                if v.1.user_id == user_id {
                    Some(v.0)
                } else {
                    None
                }
            });
            if let Some(active_player) = active_player {
                info!("active player {} disconnected ?", user_id);

                commands.entity(active_player).despawn_recursive();
                self.active_player_count -= 1;
            }
        }

        if self.player_count() == 0 {
            self.shutdown_timer.reset();
            self.shutdown_timer.unpause();
        }
    }
}

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

    if orchestration.shutdown_empty() {
        session_info.shutdown_timer.tick(time.delta());
        if session_info.shutdown_timer.finished() {
            info!("session timeout, exiting");
            exit.send(AppExit::Success);
        }
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
        if !session_info.clients.contains_key(client_id) {
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
        if !session_info.clients.contains_key(client_id) {
            warn!("client {:?} not in session", client_id);
            server.disconnect(client_id.get());
            continue;
        }

        // shared game handles the event
    }
}
