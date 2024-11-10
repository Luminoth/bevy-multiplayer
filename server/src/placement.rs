use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_mod_reqwest::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use uuid::Uuid;

use crate::{
    options::Options,
    orchestration::{start_watcher, Orchestration, StartWatcherEvent},
    server::{heartbeat, GameServerInfo, GameSessionInfo, MAX_PLAYERS},
    tasks, AppState,
};

#[derive(Debug)]
pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartWatcherEvent>()
            .add_systems(
                Update,
                (
                    update.run_if(in_state(AppState::WaitForPlacement)),
                    start_watcher,
                ),
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
                        heartbeat(
                            &mut client,
                            server_info.server_id,
                            server_info.connection_info.clone(),
                            (**state).into(),
                            None,
                        );
                    },
                );

                // have to do this with an event
                // because the runtime resource is removed while
                // the main thread callback is running
                ctx.world.send_event(StartWatcherEvent);
            }
        },
        |_ctx, err| {
            panic!("failed to ready orchestration backend: {}", err);
        },
    );
}

fn update(mut _commands: Commands, mut _app_state: ResMut<NextState<AppState>>) {
    /*warn!("faking placement!");

    let session_info = GameSessionInfo {
        session_id: Uuid::new_v4(),
        max_players: MAX_PLAYERS,
        player_session_ids: vec![],
        pending_player_ids: vec![],
    };
    info!("starting session {}", session_info.session_id);

    commands.insert_resource(session_info);

    app_state.set(AppState::InitServer);*/
}
