use bevy::prelude::*;
use clap::Parser;

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum OrchestrationType {
    Local,

    #[cfg(feature = "agones")]
    Agones,

    #[cfg(feature = "gamelift")]
    GameLift,
}

#[derive(Parser, Debug, Resource)]
pub struct Options {
    #[arg(value_enum, default_value_t = OrchestrationType::Local)]
    pub orchestration: OrchestrationType,
}
