use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Ball;

#[derive(Debug)]
pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.replicate_group::<(Transform, Ball)>();
    }
}
