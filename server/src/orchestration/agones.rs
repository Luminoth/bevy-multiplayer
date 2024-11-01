#![cfg(feature = "agones")]

use std::sync::Arc;

use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use tokio::sync::{mpsc, oneshot};

use crate::tasks;

#[derive(Clone)]
pub struct AgonesState {
    sdk: agones_api::Sdk,
    health: mpsc::Sender<()>,
    watcher: Option<Arc<oneshot::Sender<()>>>,
}

pub(super) async fn new_sdk() -> anyhow::Result<AgonesState> {
    let sdk = agones_api::Sdk::new(None, None).await?;
    let health = sdk.health_check();

    Ok(AgonesState {
        sdk,
        health,
        watcher: None,
    })
}

pub(super) async fn ready(mut agones: AgonesState) -> anyhow::Result<()> {
    info!("readying agones ...");

    agones.sdk.ready().await?;

    Ok(())
}

pub(super) fn start_watcher(mut agones: AgonesState, runtime: &TokioTasksRuntime) {
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
                    r = &mut rx => {
                        match r {
                            Ok(()) => info!("shutting down GameServer watch loop ..."),
                            // TODO: we need to shut down or something off this
                            Err(err) => error!("GameServer watch loop select error: {}", err),
                        }
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

    agones.watcher = Some(Arc::new(tx));
}

pub(super) fn stop_watcher(mut agones: AgonesState) {
    info!("stopping GameServer watch loop ...");

    agones.watcher = None;
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
