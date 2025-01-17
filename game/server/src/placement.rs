use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_tokio_tasks::TokioTasksRuntime;

use game_common::cleanup_state;

use crate::{
    is_not_headless,
    options::Options,
    orchestration::{start_watcher, Orchestration, StartWatcherEvent},
    server::{GameServerInfo, HeartbeatEvent},
    tasks, AppState,
};

#[derive(Debug, Component)]
struct OnWaitPlacement;

#[derive(Debug)]
pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartWatcherEvent>()
            .add_systems(Update, start_watcher)
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
    info!("entering placement ...");

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
                ctx.world
                    .run_system_once(|mut evw_heartbeat: EventWriter<HeartbeatEvent>| {
                        // let the backend know we're available for placement
                        evw_heartbeat.send_default();
                    })
                    .unwrap();

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
    info!("entering placement spectate ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((Camera2d, IsDefaultUiCamera, OnWaitPlacement));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..default()
            },
            Name::new("Server UI"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Server: {}", server_info.server_id)),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("Waiting for placement ..."),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));
        });
}

fn exit(mut commands: Commands) {
    info!("exiting placement ...");

    commands.remove_resource::<ClearColor>();
}
