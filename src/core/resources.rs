use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSettings {
    pub speed: f32,
    pub n_players: u8,
}
