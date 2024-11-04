use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Resource)]
pub struct PlayerClientId(pub ClientId);

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct MoveInputEvent(pub Vec2);
