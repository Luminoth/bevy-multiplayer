use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use game::{cleanup_state, AppState};

use crate::ui;

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

fn on_join_game(event: Listener<Pointer<Click>>, mut game_state: ResMut<NextState<AppState>>) {
    if !ui::check_click_event(
        event.listener(),
        event.target,
        event.button,
        PointerButton::Primary,
    ) {
        return;
    }

    //game_state.set(AppState::ConnectToServer);
    game_state.set(AppState::LoadAssets);
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

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            Name::new("Canvas"),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("Column"),
                    Pickable::IGNORE,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(150.0),
                                    height: Val::Px(50.0),
                                    border: UiRect::all(Val::Px(5.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                border_color: BorderColor(Color::BLACK),
                                border_radius: BorderRadius::MAX,
                                background_color: ui::BUTTON_NORMAL.into(),
                                ..default()
                            },
                            On::<Pointer<Click>>::run(on_join_game),
                            Name::new("Join Game"),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "Join Game",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 32.0,
                                        color: Color::srgb(0.9, 0.9, 0.9),
                                    },
                                ),
                                Pickable::IGNORE,
                            ));
                        });

                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(150.0),
                                    height: Val::Px(50.0),
                                    border: UiRect::all(Val::Px(5.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                border_color: BorderColor(Color::BLACK),
                                border_radius: BorderRadius::MAX,
                                background_color: ui::BUTTON_NORMAL.into(),
                                ..default()
                            },
                            On::<Pointer<Click>>::run(on_exit_game),
                            Name::new("Exit"),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "Exit",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 32.0,
                                        color: Color::srgb(0.9, 0.9, 0.9),
                                    },
                                ),
                                Pickable::IGNORE,
                            ));
                        });
                });
        });
}

fn exit(mut commands: Commands) {
    info!("exiting main menu ...");

    commands.remove_resource::<ClearColor>();
}
