use crate::core::constants::MAX_TERRAFORM_POINTS;
use crate::core::map::loc::Direction;
use crate::core::map::utils::rotate_bitmap;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub texture_index: usize,
    pub rotation: i32,
    pub is_base: bool,
    pub has_stone: bool,
    pub terraform: f32,
    pub visible: HashSet<usize>,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            x: 9999,
            y: 9999,
            texture_index: Self::SOIL[0],
            rotation: 0,
            is_base: false,
            has_stone: false,
            terraform: MAX_TERRAFORM_POINTS,
            visible: HashSet::new(),
        }
    }
}

impl Tile {
    pub const SIZE: f32 = 32.; // Pixel size

    pub const SIDE: u8 = 4;

    pub const SOIL: [usize; 3] = [29, 66, 67];

    pub const ANGLES: [i32; 4] = [0, 90, 180, 270];

    pub const MASKS: [u16; 69] = {
        let mut arr = [0; 69];
        arr[0] = 0b0000_0011_0111_0111;
        arr[1] = 0b0000_1111_1111_1111;
        arr[2] = 0b0000_1111_1111_1111;
        arr[3] = 0b0000_1100_1110_1110;
        arr[4] = 0b0000_0011_0111_0110;
        arr[5] = 0b0000_1100_1110_0110;
        arr[6] = 0b0111_1111_1111_0111;
        arr[7] = 0b0110_1111_1111_1111;
        arr[8] = 0b0111_0111_0111_0111;
        arr[9] = 0b1111_1111_1111_1111;
        arr[10] = 0b1111_1111_1111_1111;
        arr[11] = 0b1110_1110_1110_1110;
        arr[12] = 0b0110_0111_0011_0000;
        arr[13] = 0b0110_1110_1110_0000;
        arr[14] = 0b1110_1111_1111_1110;
        arr[15] = 0b1111_1111_1111_0110;
        arr[16] = 0b0111_0111_0111_0111;
        arr[17] = 0b1111_1111_1111_1111;
        arr[18] = 0b1111_1111_1111_1111;
        arr[19] = 0b1110_1110_1110_1110;
        arr[20] = 0b0110_0110_0110_0110;
        arr[21] = 0b0000_1111_1111_0000;
        arr[22] = 0b0111_1111_1111_1111;
        arr[23] = 0b1110_1111_1111_1111;
        arr[24] = 0b0111_0111_0011_0000;
        arr[25] = 0b1111_1111_1111_0000;
        arr[26] = 0b1111_1111_1111_0000;
        arr[27] = 0b1110_1110_1110_0000;
        arr[28] = 0b0110_1111_1111_0110;
        arr[29] = 0b0000_0000_0000_0000;
        arr[30] = 0b1111_1111_1111_0111;
        arr[31] = 0b1111_1111_1111_1110;
        arr[32] = 0b0111_1111_1111_0110;
        arr[33] = 0b0110_1111_1111_0111;
        arr[34] = 0b0110_0111_0111_0110;
        arr[35] = 0b0110_1110_1110_1110;
        arr[36] = 0b0000_1111_1111_0111;
        arr[37] = 0b0000_1111_1111_1110;
        arr[38] = 0b0000_0111_0111_0000;
        arr[39] = 0b0000_1110_1110_0000;
        arr[40] = 0b1110_1111_1111_0110;
        arr[41] = 0b0110_1111_1111_1110;
        arr[42] = 0b0111_0111_0111_0110;
        arr[43] = 0b1110_1110_1110_0110;
        arr[44] = 0b0111_1111_1111_0000;
        arr[45] = 0b1110_1111_1111_0000;
        arr[46] = 0b0000_0110_0110_0110;
        arr[47] = 0b0110_0110_0110_0000;
        arr[48] = 0b0000_0000_0001_0011;
        arr[49] = 0b0000_0011_1111_1100;
        arr[50] = 0b0000_1100_1111_0011;
        arr[51] = 0b0000_0000_1100_1110;
        arr[52] = 0b0000_0000_0000_0001;
        arr[53] = 0b0110_0110_0111_1100;
        arr[54] = 0b0110_0110_0001_0011;
        arr[55] = 0b0000_0000_0000_1000;
        arr[56] = 0b0011_0111_0110_0110;
        arr[57] = 0b1000_0000_0000_0000;
        arr[58] = 0b0001_0000_0000_0000;
        arr[59] = 0b1000_1100_0110_0110;
        arr[60] = 0b0001_1111_1110_0000;
        arr[61] = 0b1110_1100_0100_0000;
        arr[62] = 0b0011_0001_0000_0000;
        arr[63] = 0b1000_1111_0111_0000;
        arr[64] = 0b1111_1111_1111_1111;
        arr[65] = 0b1111_1111_1111_1111;
        arr[66] = 0b0000_0000_0000_0000;
        arr[67] = 0b0000_0000_0000_0000;
        arr[68] = 0b1111_1111_1111_1111;
        arr
    };

    /// Create a new tile with a random soil texture and rotation
    pub fn soil(x: u32, y: u32) -> Self {
        Tile {
            x,
            y,
            texture_index: *Self::SOIL.choose(&mut rand::rng()).unwrap(),
            rotation: *Self::ANGLES.choose(&mut rand::rng()).unwrap(),
            has_stone: rand::random::<f32>() < 0.1,
            ..default()
        }
    }

    pub fn with_stone(&self, has_stone: bool) -> Self {
        Tile {
            has_stone,
            ..self.clone()
        }
    }

    pub fn bitmap(&self) -> u16 {
        rotate_bitmap(
            Self::MASKS
                .get(self.texture_index)
                .copied()
                .unwrap_or_else(|| panic!("Invalid tile index: {}", self.texture_index)),
            self.rotation,
        )
    }

    pub fn border(&self, dir: &Direction) -> u16 {
        let bitmap = self.bitmap();
        match dir {
            Direction::North => bitmap >> 12,
            Direction::East => {
                (((bitmap >> 12) & 1) << 3)
                    | (((bitmap >> 8) & 1) << 2)
                    | (((bitmap >> 4) & 1) << 1)
                    | ((bitmap) & 1)
            }
            Direction::South => bitmap & 0b1111,
            Direction::West => {
                (((bitmap >> 15) & 1) << 3)
                    | (((bitmap >> 11) & 1) << 2)
                    | (((bitmap >> 7) & 1) << 1)
                    | ((bitmap >> 3) & 1)
            }
        }
    }

    pub fn equals(&self, other: &Tile) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn is_soil(&self) -> bool {
        Self::SOIL.contains(&self.texture_index)
    }
}
