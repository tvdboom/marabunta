use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSettings {
    pub speed: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { speed: 1.0 }
    }
}
