use bevy::prelude::*;

pub enum GameMode {
    Spectator,
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
            game_mode: GameMode::Spectator,
            speed: 1.0,
        }
    }
}
