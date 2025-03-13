use crate::core::ants::components::{Ant, AntCmp};
use crate::core::traits::Trait;
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

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: ClientId,
    pub color: AntColor,
    pub food: f32,
    pub colony: HashMap<Ant, u32>,
    pub queue: VecDeque<Ant>,
    pub traits: Vec<Trait>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            color: AntColor::Black,
            food: 100.,
            colony: HashMap::new(),
            queue: VecDeque::from([Ant::Worker, Ant::Worker, Ant::Worker]),
            traits: Vec::new(),
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

    /// Whether the player has the specified trait
    pub fn has_trait(&self, t: &Trait) -> bool {
        self.traits.contains(t)
    }

    /// Whether the player can breed this ant type
    pub fn has_ant(&self, ant: &Ant) -> bool {
        match ant {
            Ant::Alate => self.has_trait(&Trait::Alate),
            Ant::Mastodon => self.has_trait(&Trait::Mastodon),
            a if a.is_ant() => true,
            _ => false,
        }
    }
}
