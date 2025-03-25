use crate::core::ants::components::Ant;
use crate::core::constants::{ENEMY_TIMER, TRAIT_TIMER};
use crate::core::menu::settings::{AntColor, FogOfWar};
use crate::core::states::AudioState;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    SinglePlayer,
    MultiPlayer(Vec<ClientId>),
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub mode: GameMode,
    pub color: AntColor,
    pub n_opponents: u64,
    pub fog_of_war: FogOfWar,
    pub audio: AudioState,
    pub speed: f32,
    pub trait_timer: Timer,
    pub enemy_timer: Timer,
    pub termite_queue: HashMap<(u32, u32), Vec<Ant>>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mode: GameMode::SinglePlayer,
            fog_of_war: FogOfWar::default(),
            color: AntColor::default(),
            n_opponents: 1,
            audio: AudioState::default(),
            speed: 1.0,
            trait_timer: Timer::from_seconds(TRAIT_TIMER, TimerMode::Repeating),
            enemy_timer: Timer::from_seconds(ENEMY_TIMER, TimerMode::Repeating),
            termite_queue: HashMap::new(),
        }
    }
}

impl GameSettings {
    /// Reset in-game settings
    pub fn reset(&mut self) {
        self.trait_timer.reset();
        self.enemy_timer.reset();
        self.termite_queue.clear();
    }
}
