use std::collections::BTreeSet;
use std::net::{IpAddr, SocketAddr};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use serde::{Deserialize, Serialize};

use common::user::UserId;

use crate::InputState;

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

#[derive(Debug, Copy, Clone, Resource)]
pub struct PlayerClientId(ClientId);

impl PlayerClientId {
    #[inline]
    pub fn new(client_id: ClientId) -> Self {
        Self(client_id)
    }

    #[inline]
    pub fn get_client_id(&self) -> ClientId {
        self.0
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        self.0 == ClientId::SERVER
    }
}

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct ConnectEvent(pub UserId);

// TODO: add a ping event and have the client send it every 10-15 seconds
// and then have the server check for timed out clients every 30-60 seconds

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct InputUpdateEvent(pub InputState);

#[derive(Debug, Default, Event, Serialize, Deserialize)]
pub struct PlayerJumpEvent;
