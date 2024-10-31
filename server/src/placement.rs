use bevy::{ecs::system::RunSystemOnce, prelude::*};
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
    orchestration: Res<Orchestration>,
    runtime: Res<TokioTasksRuntime>,
) {
    info!("enter placement ...");

    tasks::spawn_task(
        &runtime,
        {
            let port = options.port;
            let log_paths = options.log_paths.clone();
            let orchestration = orchestration.clone();
            move || async move { orchestration.ready(port, log_paths).await }
        },
        {
            |ctx, _output| {
                ctx.world.run_system_once(
                    |mut client: BevyReqwest,
                     server_info: Res<GameServerInfo>,
                     state: Res<State<AppState>>| {
                        // let the backend know we're available for placement
                        heartbeat(&mut client, server_info.server_id, (**state).into(), None);
                    },
                );

                // TODO: can't do this because the runtime gets removed
                // before the main thread calls happen
                /*ctx.world.run_system_once(
                    |orchestration: Res<Orchestration>, runtime: Res<TokioTasksRuntime>| {
                        // TODO: need to store this sender and notify on exit
                        let _ = orchestration.start_watcher(&runtime);
                    },
                );*/
            }
        },
        |_ctx, err| {
            panic!("failed to ready orchestration backend: {}", err);
        },
    );
}

fn update(mut commands: Commands, mut app_state: ResMut<NextState<AppState>>) {
    warn!("faking placement!");

    let session_info = GameSessionInfo {
        session_id: Uuid::new_v4(),
        player_session_ids: vec![],
        pending_player_ids: vec!["test_player".into()],
    };
    info!("starting session {}", session_info.session_id);

    commands.insert_resource(session_info);

    app_state.set(AppState::InitServer);
}
