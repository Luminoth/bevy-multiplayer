use bevy::prelude::*;
use clap::Parser;

use crate::orchestration::Orchestration;

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum OrchestrationType {
    Local,

    #[cfg(feature = "agones")]
    Agones,

    #[cfg(feature = "gamelift")]
    GameLift,
}

impl OrchestrationType {
    pub fn resolve(&self) -> Orchestration {
        match self {
            Self::Local => Orchestration::Local,

            #[cfg(feature = "agones")]
            Self::Agones => Orchestration::Agones,

            #[cfg(feature = "gamelift")]
            Self::GameLift => Orchestration::GameLift,
        }
    }
}

#[derive(Parser, Debug, Resource)]
pub struct Options {
    #[arg(value_enum, default_value_t = OrchestrationType::Local)]
    pub orchestration: OrchestrationType,
}
