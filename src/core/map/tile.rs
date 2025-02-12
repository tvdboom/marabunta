use crate::core::map::utils::rotate_bitmap;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;

#[derive(Component, Clone)]
pub struct Tile {
    pub texture_index: usize,
    pub rotation: i32,
}

impl Tile {
    pub const SIZE: f32 = 32.; // Pixel size

    pub const SIDE: usize = 4;

    pub const SOIL: [usize; 3] = [29, 66, 67];

    pub fn new(texture_index: usize) -> Self {
        Self {
            texture_index,
            rotation: 0,
        }
    }

    pub fn from_rotation(texture_index: usize, rotation: i32) -> Self {
        Self {
            texture_index,
            rotation,
        }
    }

    /// Create a new tile with a random soil texture and rotation
    pub fn soil() -> Self {
        let angles = [0, 90, 180, 270];

        Tile::from_rotation(
            *Self::SOIL.choose(&mut rand::rng()).unwrap(),
            *angles.choose(&mut rand::rng()).unwrap(),
        )
    }

    pub fn bitmap(&self) -> u16 {
        rotate_bitmap(
            match self.texture_index {
                0 => 0b0000_0111_0111_0111,
                1 | 2 => 0b0000_1111_1111_1111,
                3 => 0b0000_1110_1110_1110,
                8 | 16 => 0b0111_0111_0111_0111,
                9 | 10 | 17 | 18 => 0b1111_1111_1111_1111,
                11 | 19 => 0b1110_1110_1110_1110,
                20 => 0b01111_0111_0111_0000,
                21 | 22 => 0b1111_1111_1111_0000,
                23 => 0b1110_1110_1110_0000,
                29 | 66 | 67 => 0b0000_0000_0000_0000,
                _ => 0b0000_0000_0000_0000,
            },
            self.rotation,
        )
    }
}
