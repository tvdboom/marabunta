use crate::core::ants::components::Ant;
use bevy::prelude::Resource;
use bevy_renet::renet::ClientId;

#[derive(Resource)]
pub struct Player {
    pub id: ClientId,
    pub queue: Vec<Ant>,
}

impl Player {
    pub fn new(id: ClientId) -> Self {
        Self { id, queue: vec![] }
    }
}

/// The default is used for the player in the menu
impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            queue: vec![Ant::BlackAnt; 5],
        }
    }
}
