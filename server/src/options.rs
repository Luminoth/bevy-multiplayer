use bevy::prelude::*;
use clap::Parser;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum OrchestrationType {
    Local,

    #[cfg(feature = "agones")]
    Agones,

    #[cfg(feature = "gamelift")]
    Gamelift,
}

#[derive(Parser, Debug, Resource)]
pub struct Options {
    #[arg(value_enum, default_value_t = OrchestrationType::Local)]
    pub orchestration: OrchestrationType,

    #[arg(short, long, default_value = "vec![\"logs\"]")]
    pub log_paths: Vec<String>,

    #[arg(short, long, default_value_t = 8000)]
    pub port: u16,
}
