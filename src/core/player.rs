use crate::core::ants::components::{Ant, AntCmp};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::ClientId;
use std::collections::VecDeque;

#[derive(Resource)]
pub struct Player {
    pub id: ClientId,
    pub food: f32,
    pub colony: HashMap<Ant, u32>,
    pub queue: VecDeque<Ant>,
}

impl Player {
    pub fn new(id: ClientId) -> Self {
        Self { id, ..default() }
    }

    pub fn owns(&self, ant: &AntCmp) -> bool {
        self.id == ant.owner
    }

    pub fn controls(&self, ant: &AntCmp) -> bool {
        self.id == ant.owner && !ant.kind.is_monster()
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            food: 100.,
            colony: HashMap::new(),
            queue: VecDeque::new(),
        }
    }
}
