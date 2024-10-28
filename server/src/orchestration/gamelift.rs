#![cfg(feature = "gamelift")]

use bevy::prelude::*;

pub(super) async fn ready(_api: &mut aws_gamelift_server_sdk_rs::api::Api) -> anyhow::Result<()> {
    info!("readying gamelift ...");

    // TODO: api.process_ready(...).await?;

    Ok(())
}
