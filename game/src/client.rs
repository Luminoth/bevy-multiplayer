#![cfg(not(feature = "server"))]

use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::renet::transport::{
    ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
};

use crate::{AppState, PROTOCOL_ID};

#[derive(Debug)]
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy_replicon_renet::renet::RenetClient::new(
            bevy_replicon_renet::renet::ConnectionConfig::default(),
        ))
        .add_systems(
            Update,
            (
                panic_on_network_error,
                connected.run_if(client_just_connected),
            ),
        )
        .add_systems(OnEnter(AppState::ConnectToServer), connect_to_server);
    }
}

#[allow(clippy::never_loop)]
fn panic_on_network_error(mut evt_error: EventReader<NetcodeTransportError>) {
    for evt in evt_error.read() {
        panic!("{}", evt);
    }
}

fn connect_to_server(mut commands: Commands, mut game_state: ResMut<NextState<AppState>>) {
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

    game_state.set(AppState::WaitForConnect);
}

fn connected(mut game_state: ResMut<NextState<AppState>>) {
    info!("connected!");

    game_state.set(AppState::LoadAssets);
}
