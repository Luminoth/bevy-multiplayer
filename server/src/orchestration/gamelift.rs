#![cfg(feature = "gamelift")]

use bevy::prelude::*;

pub(super) async fn ready(_api: &mut aws_gamelift_server_sdk_rs::api::Api) -> anyhow::Result<()> {
    info!("readying gamelift ...");

    // https://github.com/zamazan4ik/aws-gamelift-server-sdk-rs/blob/main/examples/basic.rs
    // TODO: api.process_ready(...).await?;

    Ok(())
}
