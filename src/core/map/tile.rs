use crate::core::map::utils::rotate_bitmap;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;

#[derive(Component, Clone)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub texture_index: usize,
    pub rotation: i32,
}

impl Tile {
    pub const SIZE: f32 = 32.; // Pixel size

    pub const SIDE: u8 = 4;

    pub const SOIL: [usize; 3] = [29, 66, 67];

    /// Create a new tile with a random soil texture and rotation
    pub fn soil(x: u32, y: u32) -> Self {
        let angles = [0, 90, 180, 270];

        Tile {
            x,
            y,
            texture_index: *Self::SOIL.choose(&mut rand::rng()).unwrap(),
            rotation: *angles.choose(&mut rand::rng()).unwrap(),
        }
    }

    pub fn bitmap(&self) -> u16 {
        rotate_bitmap(
            match self.texture_index {
                0 => 0b0000_0011_0111_0111,
                1 | 2 => 0b0000_1111_1111_1111,
                3 => 0b0000_1100_1110_1110,
                8 | 16 => 0b0111_0111_0111_0111,
                9 | 10 | 17 | 18 => 0b1111_1111_1111_1111,
                11 | 19 => 0b1110_1110_1110_1110,
                24 => 0b0111_0111_0011_0000,
                25 | 26 => 0b1111_1111_1111_0000,
                27 => 0b1110_1110_1110_0000,
                29 | 66 | 67 => 0b0000_0000_0000_0000,
                64 => 0b1111_1111_1111_1111,
                _ => panic!("Invalid tile index: {}", self.texture_index),
            },
            self.rotation,
        )
    }
}
