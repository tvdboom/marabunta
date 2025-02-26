use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    SinglePlayer,
    MultiPlayer(usize),
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub mode: GameMode,
    pub speed: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mode: GameMode::SinglePlayer,
            speed: 1.0,
        }
    }
}
