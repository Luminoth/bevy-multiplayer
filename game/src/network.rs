use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct MoveInputEvent(pub Vec2);
