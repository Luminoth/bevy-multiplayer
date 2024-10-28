mod agones;
mod gamelift;

use bevy::prelude::*;

#[derive(Resource)]
pub enum Orchestration {
    Local,

    #[cfg(feature = "agones")]
    Agones(agones_api::Sdk),

    #[cfg(feature = "gamelift")]
    GameLift(aws_gamelift_server_sdk_rs::api::Api),
}

impl Orchestration {
    pub async fn new(r#type: crate::options::OrchestrationType) -> anyhow::Result<Self> {
        match r#type {
            crate::options::OrchestrationType::Local => Ok(Self::Local),

            #[cfg(feature = "agones")]
            crate::options::OrchestrationType::Agones => {
                let sdk = agones_api::Sdk::new(None, None).await?;

                Ok(Self::Agones(sdk))
            }

            #[cfg(feature = "gamelift")]
            crate::options::OrchestrationType::GameLift => {
                let mut api = aws_gamelift_server_sdk_rs::api::Api::default();
                api.init_sdk().await?;

                Ok(Self::GameLift(api))
            }
        }
    }

    pub async fn ready(&mut self) -> anyhow::Result<()> {
        match self {
            Self::Local => {
                info!("readying local ...");
            }

            #[cfg(feature = "agones")]
            Self::Agones(sdk) => {
                agones::ready(sdk).await?;
            }

            #[cfg(feature = "gamelift")]
            Self::GameLift(api) => {
                gamelift::ready(api).await?;
            }
        }

        Ok(())
    }
}
