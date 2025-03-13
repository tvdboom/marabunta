use crate::core::ants::components::Ant;
use crate::core::constants::{ENEMY_TIMER, TRAIT_TIMER};
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
            speed: 1.0,
            audio: AudioState::default(),
            trait_timer: Timer::from_seconds(TRAIT_TIMER, TimerMode::Repeating),
            enemy_timer: Timer::from_seconds(ENEMY_TIMER, TimerMode::Repeating),
            termite_queue: HashMap::new(),
        }
    }
}
