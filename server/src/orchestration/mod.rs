mod agones;
mod gamelift;

use bevy::prelude::*;

#[derive(Debug, Resource)]
pub enum Orchestration {
    Local,

    #[cfg(feature = "agones")]
    Agones,

    #[cfg(feature = "gamelift")]
    GameLift,
}

impl Orchestration {
    pub fn ready(&self) {
        match self {
            Self::Local => {
                info!("readying local ...");
            }

            #[cfg(feature = "agones")]
            Self::Agones => {
                agones::ready();
            }

            #[cfg(feature = "gamelift")]
            Self::GameLift => {
                gamelift::ready();
            }
        }
    }
}
