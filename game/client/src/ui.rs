use bevy::{ecs::system::EntityCommands, prelude::*};

const BUTTON_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);
const BUTTON_HOVER: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.75, 0.35);

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 100.0;
const BUTTON_BORDER: f32 = 10.0;

const BUTTON_FONT: &str = "fonts/FiraSans-Bold.ttf";
const BUTTON_FONT_SIZE: f32 = 32.0;
const BUTTON_FONT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub const PICKING_BEHAVIOR_BLOCKING: PickingBehavior = PickingBehavior {
    should_block_lower: true,
    is_hoverable: false,
};

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
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Name::new(format!("Ui Canvas - {}", name.as_ref())),
        PICKING_BEHAVIOR_BLOCKING,
    ))
}

#[allow(dead_code)]
pub fn spawn_vbox<'a>(parent: &'a mut ChildBuilder) -> EntityCommands<'a> {
    parent.spawn((
        Node {
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Name::new("Column"),
        PickingBehavior::IGNORE,
    ))
}

pub fn spawn_button<'a>(
    parent: &'a mut ChildBuilder,
    asset_server: &AssetServer,
    text: impl Into<String>,
) -> EntityCommands<'a> {
    let text = text.into();

    let mut commands = parent.spawn((
        Node {
            width: Val::Px(BUTTON_WIDTH),
            height: Val::Px(BUTTON_HEIGHT),
            border: UiRect::all(Val::Px(BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Button,
        BorderColor(Color::BLACK),
        BorderRadius::MAX,
        BackgroundColor(BUTTON_NORMAL),
        Name::new(text.clone()),
    ));

    commands.with_children(|parent| {
        spawn_label(parent, asset_server, text);
    });

    commands
}

pub fn spawn_label<'a>(
    parent: &'a mut ChildBuilder,
    asset_server: &AssetServer,
    text: impl Into<String>,
) -> EntityCommands<'a> {
    parent.spawn((
        Text::new(text),
        TextFont::from_font(asset_server.load(BUTTON_FONT)).with_font_size(BUTTON_FONT_SIZE),
        TextColor(BUTTON_FONT_COLOR),
        PickingBehavior::IGNORE,
    ))
}
