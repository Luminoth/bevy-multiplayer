use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_mod_reqwest::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::renet::transport::{
    ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
};

use game::{cleanup_state, GameState, PROTOCOL_ID};

use crate::{ui, AppState};

#[derive(Debug, Component)]
struct OnConnectServer;

#[derive(Debug)]
pub struct ConnectServerPlugin;

impl Plugin for ConnectServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ConnectToServer), enter)
            .add_systems(
                Update,
                (
                    panic_on_network_error,
                    connected.run_if(client_just_connected),
                ),
            )
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

// TODO: remove this and actually handle the errors
#[allow(clippy::never_loop)]
fn panic_on_network_error(mut evt_error: EventReader<NetcodeTransportError>) {
    for evt in evt_error.read() {
        panic!("{}", evt);
    }
}

fn on_cancel(event: Listener<Pointer<Click>>, mut app_state: ResMut<NextState<AppState>>) {
    if !ui::check_click_event(
        event.listener(),
        event.target,
        event.button,
        PointerButton::Primary,
    ) {
        return;
    }

    app_state.set(AppState::MainMenu);
}

fn enter(mut commands: Commands, asset_server: Res<AssetServer>, _client: BevyReqwest) {
    info!("entering connect server ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((
        Camera2dBundle::default(),
        IsDefaultUiCamera,
        OnConnectServer,
    ));

    ui::spawn_canvas(&mut commands, "Connect Server").with_children(|parent| {
        ui::spawn_vbox(parent).with_children(|parent| {
            ui::spawn_label(parent, &asset_server, "Connecting to server ...");

            ui::spawn_button(
                parent,
                &asset_server,
                "Cancel",
                On::<Pointer<Click>>::run(on_cancel),
            );
        });
    });

    //api::find_server(client, "test_player");
    connect_to_server(commands);
}

fn exit(mut commands: Commands) {
    info!("exiting connect server ...");

    commands.remove_resource::<ClearColor>();
}

fn connect_to_server(mut commands: Commands) {
    info!("connect to server ...");

    let server_addr = "127.0.0.1:5576".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    info!("connecting to {} ...", server_addr);

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    commands.insert_resource(transport);
}

fn connected(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    info!("connected!");

    app_state.set(AppState::InGame);
    game_state.set(GameState::LoadAssets);
}
