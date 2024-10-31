#![cfg(feature = "agones")]

use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use tokio::sync::{mpsc, oneshot};

use crate::tasks;

#[derive(Clone)]
pub struct AgonesState {
    sdk: agones_api::Sdk,
    health: mpsc::Sender<()>,
}

pub(super) async fn new_sdk() -> anyhow::Result<AgonesState> {
    let sdk = agones_api::Sdk::new(None, None).await?;
    let health = sdk.health_check();

    Ok(AgonesState { sdk, health })
}

pub(super) async fn ready(mut agones: AgonesState) -> anyhow::Result<()> {
    info!("readying agones ...");

    agones.sdk.ready().await?;

    Ok(())
}

#[must_use]
pub(super) fn start_watcher(
    agones: AgonesState,
    runtime: &TokioTasksRuntime,
) -> oneshot::Sender<()> {
    let mut watch_client = agones.sdk.clone();
    let (tx, mut rx) = oneshot::channel::<()>();
    tasks::spawn_task(
        runtime,
        move || async move {
            info!("starting GameServer watch loop ...");

            let mut stream = watch_client.watch_gameserver().await?;
            loop {
                tokio::select! {
                    gs = stream.message() => {
                        match gs {
                            Ok(Some(gs)) => {
                                info!("GameServer Update, name: {}", gs.object_meta.unwrap().name);
                                info!("GameServer Update, state: {}", gs.status.unwrap().state);
                            }
                            Ok(None) => {
                                info!("server closed the GameServer watch stream");
                                break;
                            }
                            Err(err) => {
                                // TODO: this probably should do something ...
                                error!("GameServer Update stream encountered an error: {}", err);
                            }
                        }

                    }
                    _ = &mut rx => {
                        info!("shutting down GameServer watch loop ...");
                        break;
                    }
                }
            }

            Ok(())
        },
        |_ctx, _output| {},
        |_ctx, err| {
            // TODO: we need to shut down or something off this
            error!("failed to watch for GameServer updates: {}", err);
        },
    );

    tx
}

pub(super) async fn health_check(agones: AgonesState) -> anyhow::Result<()> {
    debug!("health checking agones ...");

    agones.health.send(()).await?;

    Ok(())
}

pub(super) async fn shutdown(mut agones: AgonesState) -> anyhow::Result<()> {
    info!("shutdown agones ...");

    agones.sdk.shutdown().await?;

    Ok(())
}
