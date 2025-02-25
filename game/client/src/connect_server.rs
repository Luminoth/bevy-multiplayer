use std::net::UdpSocket;

use bevy::prelude::*;
use bevy_mod_reqwest::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::{ConnectionConfig, RenetClient},
    RenetChannelsExt,
};

use common::gameclient::*;
use game_common::{
    cleanup_state,
    network::{ConnectEvent, PlayerClientId},
    utils::current_timestamp,
    PROTOCOL_ID,
};

use crate::{api, client, options::Options, ui, AppState};

#[derive(Debug, Component)]
struct Status;

#[derive(Debug, Component)]
struct OnConnectServer;

#[derive(Debug)]
pub struct ConnectServerPlugin;

impl Plugin for ConnectServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ConnectToServer), enter)
            .add_systems(Update, connected.run_if(client_just_connected))
            .add_systems(
                OnExit(AppState::ConnectToServer),
                (
                    exit,
                    cleanup_state::<OnConnectServer>,
                    cleanup_state::<Node>,
                ),
            );
    }
}

fn on_cancel(trigger: Trigger<Pointer<Click>>, mut app_state: ResMut<NextState<AppState>>) {
    if trigger.button == PointerButton::Primary {
        app_state.set(AppState::MainMenu);
    }
}

fn on_find_server(
    req: Trigger<ReqwestResponseEvent>,
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    mut status_query: Query<&mut Text, With<Status>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    let resp = req.event().as_str().unwrap();

    let resp: FindServerResponseV1 = serde_json::from_str(resp).unwrap();
    if resp.address.is_empty() {
        error!("find server failed");
        app_state.set(AppState::MainMenu);
        return;
    }

    connect_to_server(
        &mut commands,
        &channels,
        resp.address,
        resp.port,
        &mut status_query,
    );
}

fn on_find_server_error(
    trigger: Trigger<ReqwestErrorEvent>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    let e = &trigger.event().0;
    error!("find server error: {:?}", e);

    app_state.set(AppState::MainMenu);
}

fn enter(
    mut commands: Commands,
    options: Res<Options>,
    asset_server: Res<AssetServer>,
    mut client: BevyReqwest,
) {
    info!("entering connect server ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((Camera2d, IsDefaultUiCamera, OnConnectServer));

    ui::spawn_canvas(&mut commands, "Connect Server").with_children(|parent| {
        ui::spawn_label(parent, &asset_server, "Finding server ...").insert(Status);
        ui::spawn_button(parent, &asset_server, "Cancel").observe(on_cancel);
    });

    api::find_server(&mut client, options.user_id)
        .unwrap()
        .on_response(on_find_server)
        .on_error(on_find_server_error);
}

fn exit(mut commands: Commands) {
    info!("exiting connect server ...");

    commands.remove_resource::<ClearColor>();
}

fn connect_to_server(
    commands: &mut Commands,
    channels: &RepliconChannels,
    address: impl AsRef<str>,
    port: u16,
    status_query: &mut Query<&mut Text, With<Status>>,
) {
    info!("connect to server ...");

    status_query.single_mut().0 = "Connecting to server ...".to_owned();

    let address = address.as_ref();
    let server_addr = format!("{}:{}", address, port).parse().unwrap();
    let socket = UdpSocket::bind(format!("{}:0", address)).unwrap();
    let current_time = current_timestamp();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    info!("connecting to {} as {} ...", server_addr, client_id);

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config: channels.get_server_configs(),
        client_channels_config: channels.get_client_configs(),
        ..Default::default()
    });
    commands.insert_resource(client);

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    commands.insert_resource(transport);

    commands.insert_resource(client::ClientState::new_remote(address));
    commands.insert_resource(PlayerClientId::new(ClientId::new(client_id)));
}

fn connected(
    options: Res<Options>,
    client: Res<client::ClientState>,
    client_id: Res<PlayerClientId>,
    mut evw_connect: EventWriter<ConnectEvent>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    info!("connected to server!");

    client::on_connected_server(
        &client,
        *client_id,
        options.user_id,
        &mut evw_connect,
        &mut app_state,
    );
}
