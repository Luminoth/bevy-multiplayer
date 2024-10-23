use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_mod_picking::prelude::*;

const BUTTON_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);
const BUTTON_HOVER: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.75, 0.35);

const BUTTON_WIDTH: f32 = 150.0;
const BUTTON_HEIGHT: f32 = 50.0;
const BUTTON_BORDER: f32 = 5.0;

const BUTTON_FONT: &str = "fonts/FiraSans-Bold.ttf";
const BUTTON_FONT_SIZE: f32 = 32.0;
const BUTTON_FONT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[inline]
pub fn check_click_event(
    listener: Entity,
    target: Entity,
    event_button: PointerButton,
    expected_button: PointerButton,
) -> bool {
    target == listener && event_button == expected_button
}

#[derive(Debug)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_button);
    }
}

#[allow(clippy::type_complexity)]
fn update_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_NORMAL.into();
            }
        }
    }
}

pub fn spawn_canvas<'a>(commands: &'a mut Commands, name: impl AsRef<str>) -> EntityCommands<'a> {
    commands.spawn((
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
        Name::new(format!("Ui Canvas - {}", name.as_ref())),
        Pickable::IGNORE,
    ))
}

pub fn spawn_vbox<'a>(parent: &'a mut ChildBuilder) -> EntityCommands<'a> {
    parent.spawn((
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
}

pub fn spawn_button(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    text: impl Into<String>,
    on_click: On<Pointer<Click>>,
) {
    let text = text.into();

    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(BUTTON_WIDTH),
                    height: Val::Px(BUTTON_HEIGHT),
                    border: UiRect::all(Val::Px(BUTTON_BORDER)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                border_radius: BorderRadius::MAX,
                background_color: BUTTON_NORMAL.into(),
                ..default()
            },
            on_click,
            Name::new(text.clone()),
        ))
        .with_children(|parent| {
            spawn_label(parent, asset_server, text);
        });
}

pub fn spawn_label(parent: &mut ChildBuilder, asset_server: &AssetServer, text: impl Into<String>) {
    parent.spawn((
        TextBundle::from_section(
            text,
            TextStyle {
                font: asset_server.load(BUTTON_FONT),
                font_size: BUTTON_FONT_SIZE,
                color: BUTTON_FONT_COLOR,
            },
        ),
        Pickable::IGNORE,
    ));
}
