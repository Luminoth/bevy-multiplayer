use bevy::prelude::*;
use clap::Parser;

use common::user::UserId;

#[derive(Parser, Debug, Resource)]
pub struct Options {
    #[arg(default_value_t = UserId::new_v4())]
    pub user_id: UserId,
}
