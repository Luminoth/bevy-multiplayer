#![cfg(feature = "agones")]

use std::sync::Arc;

use bevy::prelude::*;
use tokio::sync::RwLock;

pub type AgonesSdk = Arc<RwLock<agones_api::Sdk>>;

pub(super) async fn new_sdk() -> anyhow::Result<AgonesSdk> {
    let sdk = agones_api::Sdk::new(None, None).await?;

    Ok(Arc::new(RwLock::new(sdk)))
}

pub(super) async fn ready(sdk: AgonesSdk) -> anyhow::Result<()> {
    info!("readying agones ...");

    sdk.write().await.ready().await?;

    Ok(())
}
