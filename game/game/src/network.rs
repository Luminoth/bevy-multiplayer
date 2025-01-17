use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use common::user::UserId;

use crate::InputState;

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
