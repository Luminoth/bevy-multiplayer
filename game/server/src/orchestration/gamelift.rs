#![cfg(feature = "gamelift")]

use std::sync::Arc;

use aws_gamelift_server_sdk_rs::{
    log_parameters::LogParameters, process_parameters::ProcessParameters,
};
use bevy::prelude::*;
use tokio::sync::RwLock;

pub type GameliftApi = Arc<RwLock<aws_gamelift_server_sdk_rs::api::Api>>;

pub(super) async fn new_api() -> anyhow::Result<GameliftApi> {
    let mut api = aws_gamelift_server_sdk_rs::api::Api::default();
    api.init_sdk().await?;

    Ok(Arc::new(RwLock::new(api)))
}

pub(super) async fn ready(
    api: GameliftApi,
    port: u16,
    log_paths: Vec<String>,
) -> anyhow::Result<()> {
    info!("readying gamelift ...");

    api.write()
        .await
        .process_ready(ProcessParameters {
            on_start_game_session: Box::new({
                let api = api.clone();
                move |game_session| {
                    Box::pin({
                        let api = api.clone();
                        async move {
                            debug!("{:?}", game_session);

                            api.write().await.activate_game_session().await.unwrap();

                            info!("session active!");
                        }
                    })
                }
            }),
            on_update_game_session: Box::new(|update_game_session| {
                Box::pin(async move { debug!("{:?}", update_game_session) })
            }),
            on_process_terminate: Box::new(|| Box::pin(async {})),
            on_health_check: Box::new(|| Box::pin(async { true })),
            port: port as i32,
            log_parameters: LogParameters { log_paths },
        })
        .await?;

    Ok(())
}
