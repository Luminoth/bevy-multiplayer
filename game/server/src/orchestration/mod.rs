mod agones;
mod gamelift;

use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;

use common::gameserver::GameServerOrchestration;

#[derive(Debug, Event)]
pub struct StartWatcherEvent;

pub fn start_watcher(
    mut evt: EventReader<StartWatcherEvent>,
    orchestration: Option<Res<Orchestration>>,
    runtime: Res<TokioTasksRuntime>,
) {
    if evt.is_empty() {
        return;
    }
    evt.clear();

    info!("starting orchestration watcher ...");

    orchestration.unwrap().start_watcher(&runtime);
}

#[derive(Clone, Resource)]
pub enum Orchestration {
    Local,

    #[cfg(feature = "agones")]
    Agones(agones::AgonesState),

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
            crate::options::OrchestrationType::GameLift => {
                Ok(Self::GameLift(gamelift::new_api().await?))
            }
        }
    }

    #[inline]
    pub fn shutdown_empty(&self) -> bool {
        match self {
            Self::Local => false,

            #[cfg(feature = "agones")]
            Self::Agones(_) => true,

            #[cfg(feature = "gamelift")]
            Self::GameLift(_) => true,
        }
    }

    #[inline]
    pub fn as_api_type(&self) -> GameServerOrchestration {
        match self {
            Self::Local => GameServerOrchestration::Local,

            #[cfg(feature = "agones")]
            Self::Agones(_) => GameServerOrchestration::Agones,

            #[cfg(feature = "gamelift")]
            Self::GameLift(_) => GameServerOrchestration::GameLift,
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

    pub fn start_watcher(&self, runtime: &TokioTasksRuntime) {
        match self {
            #[cfg(feature = "agones")]
            Self::Agones(sdk) => agones::start_watcher(sdk.clone(), runtime),

            _ => (),
        }
    }

    pub fn stop_watcher(&self) {
        match self {
            #[cfg(feature = "agones")]
            Self::Agones(sdk) => agones::stop_watcher(sdk.clone()),

            _ => (),
        }
    }

    pub async fn health_check(&self) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "agones")]
            Self::Agones(sdk) => {
                agones::health_check(sdk.clone()).await?;
            }

            _ => (),
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "agones")]
            Self::Agones(sdk) => {
                agones::shutdown(sdk.clone()).await?;
            }

            _ => (),
        }

        Ok(())
    }
}
