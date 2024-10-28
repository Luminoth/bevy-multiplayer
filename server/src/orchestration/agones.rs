#![cfg(feature = "agones")]

use bevy::prelude::*;

pub(super) async fn ready(sdk: &mut agones_api::Sdk) -> anyhow::Result<()> {
    info!("readying agones ...");

    sdk.ready().await?;

    Ok(())
}
