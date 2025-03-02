use crate::core::ants::components::Ant;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;

#[derive(Resource)]
pub struct Player {
    pub id: ClientId,
    pub food: f32,
    pub queue: Vec<Ant>,
}

impl Player {
    pub fn new(id: ClientId) -> Self {
        Self {
            id,
            food: 100.,
            queue: vec![],
        }
    }
}

/// The default is used for the player in the menu
impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            queue: vec![Ant::BlackAnt, Ant::BlackBullet, Ant::BlackSoldier],
            ..default()
        }
    }
}
