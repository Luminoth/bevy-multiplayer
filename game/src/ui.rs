#![cfg(not(feature = "server"))]

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

pub const BUTTON_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BUTTON_HOVER: Color = Color::srgb(0.25, 0.25, 0.25);
pub const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.75, 0.35);

#[inline]
pub fn check_click_event(
    listener: Entity,
    target: Entity,
    event_button: PointerButton,
    expected_button: PointerButton,
) -> bool {
    target == listener && event_button == expected_button
}

#[allow(clippy::type_complexity)]
pub fn update_button(
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
