use bevy::prelude::*;
use bevy_mod_reqwest::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use uuid::Uuid;

use crate::{
    orchestration::Orchestration,
    server::{heartbeat, GameServerInfo, GameSessionInfo},
    AppState,
};

#[derive(Debug)]
pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update.run_if(in_state(AppState::WaitForPlacement)),),
        )
        .add_systems(OnEnter(AppState::WaitForPlacement), enter);
    }
}

fn enter(orchestration: ResMut<Orchestration>, runtime: ResMut<TokioTasksRuntime>) {
    info!("enter placement ...");

    runtime.spawn_background_task({
        let mut orchestration = orchestration.clone();
        |mut ctx| async move {
            let result = orchestration.ready().await;
            ctx.run_on_main_thread(move |_| {
                result.unwrap();

                // TODO: send a heartbeat to update our "state"
            })
            .await;
        }
    });
}

fn update(
    mut commands: Commands,
    mut client: BevyReqwest,
    server_info: Res<GameServerInfo>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    warn!("faking placement!");

    let session_info = GameSessionInfo {
        session_id: Uuid::new_v4(),
        player_session_ids: vec![],
        pending_player_ids: vec!["test_player".into()],
    };
    info!("starting session {}", session_info.session_id);

    heartbeat(&mut client, &server_info, Some(&session_info));

    commands.insert_resource(session_info);

    app_state.set(AppState::InitServer);
}
