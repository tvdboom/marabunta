use crate::core::ants::components::Ant;
use crate::core::menu::settings::AntColor;
use crate::core::resources::Resources;
use crate::core::traits::Trait;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Players(pub Vec<Player>);

impl Default for Players {
    fn default() -> Self {
        Self(vec![Player::default()])
    }
}

impl Players {
    pub fn get(&self, id: ClientId) -> &Player {
        self.0
            .iter()
            .find(|p| p.id == id || p.id == ClientId::MAX)
            .unwrap()
    }

    pub fn get_mut(&mut self, id: ClientId) -> &mut Player {
        self.0
            .iter_mut()
            .find(|p| p.id == id || p.id == ClientId::MAX)
            .unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: ClientId,
    pub color: AntColor,
    pub resources: Resources,
    pub visible_tiles: HashSet<(u32, u32)>,
    pub queue: VecDeque<Ant>,
    pub traits: Vec<Trait>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: ClientId::MAX,
            color: AntColor::Black,
            resources: Resources {
                leaves: 150.,
                nutrients: 0.,
            },
            visible_tiles: HashSet::new(),
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
