use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Player {
    pub id: usize,
}

impl Player {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}
