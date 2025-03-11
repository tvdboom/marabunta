use crate::core::ants::components::{Ant, AntCmp};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AntColor {
    Black,
    Red,
}

#[derive(Resource)]
pub struct Player {
    pub id: ClientId,
    pub color: AntColor,
    pub food: f32,
    pub colony: HashMap<Ant, u32>,
    pub queue: VecDeque<Ant>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            color: AntColor::Black,
            food: 100.,
            colony: HashMap::new(),
            queue: VecDeque::new(),
        }
    }
}

impl Player {
    pub fn new(id: ClientId, color: AntColor) -> Self {
        Self {
            id,
            color,
            ..default()
        }
    }

    /// Whether the player owns the ant (includes monsters)
    pub fn owns(&self, ant: &AntCmp) -> bool {
        self.id == ant.owner
    }

    /// Whether the player controls the ant (own colony)
    pub fn controls(&self, ant: &AntCmp) -> bool {
        self.id == ant.owner && ant.kind.is_ant()
    }
}
