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

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct InputUpdateEvent(pub InputState);

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct PlayerJumpEvent;
