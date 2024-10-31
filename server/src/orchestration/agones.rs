#![cfg(feature = "agones")]

use bevy::prelude::*;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct AgonesState {
    sdk: agones_api::Sdk,
    health: Sender<()>,
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
