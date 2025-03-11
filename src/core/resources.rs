use crate::core::ants::components::AntCmp;
use crate::core::constants::TRAIT_TIMER;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PopulationT = HashMap<Uuid, (Transform, AntCmp)>;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    SinglePlayer,
    MultiPlayer(Vec<ClientId>),
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub mode: GameMode,
    pub speed: f32,
    pub trait_timer: Timer,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mode: GameMode::SinglePlayer,
            speed: 1.0,
            trait_timer: Timer::from_seconds(TRAIT_TIMER, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub struct Population(pub PopulationT);

impl Default for Population {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
