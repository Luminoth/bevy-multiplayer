use bevy::prelude::*;

#[derive(Debug, Resource, Reflect)]
pub struct Settings {
    pub look_sensitivity: f32,

    pub invert_look: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            look_sensitivity: 4.0,
            invert_look: true, //false,
        }
    }
}
