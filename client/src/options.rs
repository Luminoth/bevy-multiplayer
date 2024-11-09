use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Debug, Resource)]
pub struct Options {
    pub player_id: String,
}
