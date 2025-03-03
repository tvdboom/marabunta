use crate::core::ants::components::Ant;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::ClientId;

#[derive(Resource)]
pub struct Player {
    pub id: ClientId,
    pub food: f32,
    pub colony: HashMap<Ant, u32>,
    pub queue: Vec<Ant>,
}

impl Player {
    pub fn new(id: ClientId) -> Self {
        Self {
            id,
            queue: vec![],
            ..default()
        }
    }
}

/// The default is used for the player in the menu
impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            food: 100.,
            colony: HashMap::new(),
            queue: vec![Ant::BlackAnt, Ant::BlackBullet, Ant::BlackBullet],
        }
    }
}
