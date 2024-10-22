#![cfg(feature = "server")]

use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_replicon_renet::renet::transport::{
    NetcodeServerTransport, ServerAuthentication, ServerConfig,
};
use bevy_replicon_renet::renet::ServerEvent;

use crate::AppState;

pub fn wait_for_placement(mut game_state: ResMut<NextState<AppState>>) {
    warn!("faking placement!");
    game_state.set(AppState::InitServer);
}

pub fn init_network(mut commands: Commands, mut game_state: ResMut<NextState<AppState>>) {
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
        protocol_id: crate::PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    info!("listening at {} ...", server_addr);

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    commands.insert_resource(transport);

    game_state.set(AppState::LoadAssets);
}

pub fn shutdown_network(mut commands: Commands) {
    info!("shutdown network ...");

    commands.remove_resource::<NetcodeServerTransport>();
}

pub fn handle_network_events(mut evt_server: EventReader<ServerEvent>) {
    for evt in evt_server.read() {
        match evt {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client {} connected", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {} disconnected: {}", client_id, reason);
            }
        }
    }
}
