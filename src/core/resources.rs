use bevy::prelude::*;

pub enum GameMode {
    SinglePlayer,
    MultiPlayer,
}

#[derive(Resource)]
pub struct GameSettings {
    pub game_mode: GameMode,
    pub speed: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            game_mode: GameMode::SinglePlayer,
            speed: 1.0,
        }
    }
}
