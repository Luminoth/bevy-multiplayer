mod agones;
mod gamelift;

use bevy::prelude::*;

#[derive(Clone, Resource)]
pub enum Orchestration {
    Local,

    #[cfg(feature = "agones")]
    Agones(agones::AgonesSdk),

    #[cfg(feature = "gamelift")]
    GameLift(gamelift::GameliftApi),
}

impl Orchestration {
    pub async fn new(r#type: crate::options::OrchestrationType) -> anyhow::Result<Self> {
        match r#type {
            crate::options::OrchestrationType::Local => Ok(Self::Local),

            #[cfg(feature = "agones")]
            crate::options::OrchestrationType::Agones => Ok(Self::Agones(agones::new_sdk().await?)),

            #[cfg(feature = "gamelift")]
            crate::options::OrchestrationType::Gamelift => {
                Ok(Self::GameLift(gamelift::new_api().await?))
            }
        }
    }

    pub async fn ready(&self, port: u16, log_paths: Vec<String>) -> anyhow::Result<()> {
        match self {
            Self::Local => {
                info!("readying local ...");
            }

            #[cfg(feature = "agones")]
            Self::Agones(sdk) => {
                agones::ready(sdk.clone()).await?;
            }

            #[cfg(feature = "gamelift")]
            Self::GameLift(api) => {
                gamelift::ready(api.clone(), port, log_paths.clone()).await?;
            }
        }

        Ok(())
    }
}
