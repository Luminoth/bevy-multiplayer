use bevy::prelude::*;

#[derive(Debug, Reflect)]
pub struct MnkSettings {
    pub mouse_sensitivity: f32,

    pub invert_look: bool,
}

impl Default for MnkSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.5,
            invert_look: false,
        }
    }
}

#[derive(Debug, Reflect)]
pub struct GamepadSettings {
    pub look_sensitivity: f32,

    pub invert_look: bool,
}

impl Default for GamepadSettings {
    fn default() -> Self {
        Self {
            look_sensitivity: 4.0,
            invert_look: true, //false,
        }
    }
}

#[derive(Debug, Default, Resource, Reflect)]
pub struct Settings {
    pub mnk: MnkSettings,
    pub gamepad: GamepadSettings,
}
