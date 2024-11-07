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
    #[arg(long)]
    pub headless: bool,

    #[arg(value_enum, default_value_t = OrchestrationType::Local)]
    pub orchestration: OrchestrationType,

    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    #[arg(short, long, default_value_t = 5576)]
    pub port: u16,

    #[arg(short, long, default_value = "vec![\"logs\"]")]
    pub log_paths: Vec<String>,
}

impl Options {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
