use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use game::{cleanup_state, GameState};

use crate::{ui, AppState};

#[derive(Debug, Component)]
struct OnMainMenu;

#[derive(Debug)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), enter)
            .add_systems(
                OnExit(AppState::MainMenu),
                (exit, cleanup_state::<OnMainMenu>, cleanup_state::<Node>),
            );
    }
}

fn on_find_game(
    event: Listener<Pointer<Click>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if !ui::check_click_event(
        event.listener(),
        event.target,
        event.button,
        PointerButton::Primary,
    ) {
        return;
    }

    //app_state.set(AppState::ConnectToServer);
    app_state.set(AppState::InGame);
    game_state.set(GameState::LoadAssets);
}

fn on_exit_game(event: Listener<Pointer<Click>>, mut exit: EventWriter<AppExit>) {
    if !ui::check_click_event(
        event.listener(),
        event.target,
        event.button,
        PointerButton::Primary,
    ) {
        return;
    }

    exit.send(AppExit::Success);
}

fn enter(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("entering main menu ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera, OnMainMenu));

    ui::spawn_canvas(&mut commands, "Main Menu").with_children(|parent| {
        ui::spawn_vbox(parent).with_children(|parent| {
            ui::spawn_button(
                parent,
                &asset_server,
                "Find Game",
                On::<Pointer<Click>>::run(on_find_game),
            );

            ui::spawn_button(
                parent,
                &asset_server,
                "Exit",
                On::<Pointer<Click>>::run(on_exit_game),
            );
        });
    });
}

fn exit(mut commands: Commands) {
    info!("exiting main menu ...");

    commands.remove_resource::<ClearColor>();
}
