mod agones;
mod gamelift;

use std::sync::Arc;

use bevy::prelude::*;
use tokio::sync::RwLock;

#[derive(Clone, Resource)]
pub enum Orchestration {
    Local,

    #[cfg(feature = "agones")]
    Agones(Arc<RwLock<agones_api::Sdk>>),

    #[cfg(feature = "gamelift")]
    GameLift(Arc<RwLock<aws_gamelift_server_sdk_rs::api::Api>>),
}

impl Orchestration {
    pub async fn new(r#type: crate::options::OrchestrationType) -> anyhow::Result<Self> {
        match r#type {
            crate::options::OrchestrationType::Local => Ok(Self::Local),

            #[cfg(feature = "agones")]
            crate::options::OrchestrationType::Agones => {
                let sdk = agones_api::Sdk::new(None, None).await?;

                Ok(Self::Agones(Arc::new(RwLock::new(sdk))))
            }

            #[cfg(feature = "gamelift")]
            crate::options::OrchestrationType::GameLift => {
                let mut api = aws_gamelift_server_sdk_rs::api::Api::default();
                api.init_sdk().await?;

                Ok(Self::GameLift(Arc::new(RwLock::new(api))))
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
                let mut sdk = sdk.write().await;
                agones::ready(&mut sdk).await?;
            }

            #[cfg(feature = "gamelift")]
            Self::GameLift(api) => {
                let mut api = api.write().await;
                gamelift::ready(&mut api).await?;
            }
        }

        Ok(())
    }
}
