use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use game_common::GameState;

use crate::{ui, AppState};

#[derive(Debug, Component)]
pub struct GameMenu;

#[derive(Debug, Default)]
pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup);
    }
}

fn on_quit_game(
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

    app_state.set(AppState::MainMenu);
    game_state.set(GameState::WaitingForApp);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("creating pause menu ...");

    // TODO: this canvas should be transparent grey
    ui::spawn_canvas(&mut commands, "Pause Menu")
        .insert(GameMenu)
        .insert(Visibility::Hidden)
        .with_children(|parent| {
            ui::spawn_button(
                parent,
                &asset_server,
                "Quit Game",
                On::<Pointer<Click>>::run(on_quit_game),
            );
        });
}
