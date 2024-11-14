use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_mod_reqwest::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use uuid::Uuid;

use game_common::cleanup_state;

use crate::{
    is_not_headless,
    options::Options,
    orchestration::{start_watcher, Orchestration, StartWatcherEvent},
    server::{heartbeat, GameServerInfo, GameSessionInfo, MAX_PLAYERS},
    tasks, AppState,
};

#[derive(Debug, Component)]
struct OnWaitPlacement;

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
            .add_systems(
                OnEnter(AppState::WaitForPlacement),
                (enter, enter_spectate.run_if(is_not_headless)),
            )
            .add_systems(
                OnExit(AppState::WaitForPlacement),
                (
                    exit,
                    cleanup_state::<OnWaitPlacement>,
                    cleanup_state::<Node>,
                ),
            );
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
                     state: Res<State<AppState>>,
                     orchestration: Res<Orchestration>| {
                        // let the backend know we're available for placement
                        heartbeat(
                            &mut client,
                            server_info.server_id,
                            server_info.connection_info.clone(),
                            (**state).into(),
                            orchestration.as_api_type(),
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

fn enter_spectate(mut commands: Commands, server_info: Res<GameServerInfo>) {
    info!("enter placement spectate ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((
        Camera2dBundle::default(),
        IsDefaultUiCamera,
        OnWaitPlacement,
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
            Name::new("Server UI"),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                format!("Server: {}", server_info.server_id),
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent.spawn(TextBundle::from_section(
                "Waiting for placement ...",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn exit(mut commands: Commands) {
    info!("exiting placement ...");

    commands.remove_resource::<ClearColor>();
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
