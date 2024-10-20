#![cfg(not(feature = "server"))]

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const _HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const _PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[inline]
pub fn check_click_event(
    listener: Entity,
    target: Entity,
    event_button: PointerButton,
    expected_button: PointerButton,
) -> bool {
    target == listener && event_button == expected_button
}
