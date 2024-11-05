use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::InputState;

#[derive(Debug, Resource)]
pub struct PlayerClientId(pub ClientId);

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct InputUpdateEvent(pub InputState);

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct PlayerJumpEvent;
