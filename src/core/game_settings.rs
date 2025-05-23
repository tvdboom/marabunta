use crate::core::ants::components::Ant;
use crate::core::constants::TRAIT_TIMER;
use crate::core::menu::settings::{AntColor, Background, FogOfWar};
use crate::core::states::AudioState;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    SinglePlayer,
    Multiplayer,
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub game_mode: GameMode,
    pub color: AntColor,
    pub background: Background,
    pub npcs: u64,
    pub fog_of_war: FogOfWar,
    pub audio: AudioState,
    pub speed: f32,
    pub trait_timer: Timer,
    pub termite_queue: HashMap<(u32, u32), Vec<Ant>>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            game_mode: GameMode::SinglePlayer,
            fog_of_war: FogOfWar::default(),
            background: Background::default(),
            color: AntColor::default(),
            npcs: 1,
            audio: AudioState::default(),
            speed: 1.0,
            trait_timer: Timer::from_seconds(TRAIT_TIMER, TimerMode::Repeating),
            termite_queue: HashMap::new(),
        }
    }
}

impl GameSettings {
    /// Reset in-game settings
    pub fn reset(&mut self) {
        self.trait_timer.reset();
        self.termite_queue.clear();
    }
}
