use crate::core::ants::components::AntCmp;
use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Player {
    pub id: usize,
    pub queue: Vec<AntCmp>,
}

impl Player {
    pub fn new(id: usize) -> Self {
        Self { id, queue: vec![] }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new(0)
    }
}