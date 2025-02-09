use bevy::prelude::Component;

#[derive(Component, Clone)]
pub struct Tile {
    pub index: usize,
    pub bitmap: u16,
}

impl Tile {
    pub const SIZE: f32 = 32.; // Pixel size

    pub fn new(index: usize) -> Self {
        Self {
            index,
            bitmap: match index {
                0 => 0b0000_0000,
                _ => 0b0000_0000,
            },
        }
    }
}
