use bevy::prelude::*;
use bevy_mod_reqwest::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use uuid::Uuid;

use crate::{
    options::Options,
    orchestration::Orchestration,
    server::{heartbeat, GameServerInfo, GameSessionInfo},
    tasks, AppState,
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

fn enter(
    options: Res<Options>,
    orchestration: ResMut<Orchestration>,
    mut runtime: ResMut<TokioTasksRuntime>,
) {
    info!("enter placement ...");

    let port = options.port;
    let log_paths = options.log_paths.clone();
    let orchestration = orchestration.clone();
    tasks::spawn_task(
        &mut runtime,
        move || async move { orchestration.ready(port, log_paths).await },
        |_ctx, _output| {
            // TODO: send a heartbeat to update our "state"
        },
        |_ctx, err| {
            panic!("failed to ready orchestration backend: {}", err);
        },
    );
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
