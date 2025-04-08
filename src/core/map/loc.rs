use crate::core::map::map::Map;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub const CARDINALS: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::NorthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::SouthWest,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
        }
    }

    pub fn degrees(&self) -> f32 {
        match self {
            Direction::North => 0.,
            Direction::NorthEast => PI * 0.25,
            Direction::East => -PI * 0.5,
            Direction::SouthEast => -PI * 0.75,
            Direction::South => PI,
            Direction::SouthWest => PI * 1.25,
            Direction::West => PI * 0.5,
            Direction::NorthWest => PI * 0.75,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Loc {
    pub x: u32,
    pub y: u32,
    pub bit: u8, // 0-15, representing a position in the tile's 4x4 bit grid
}

impl Loc {
    pub fn is_map_edge(&self) -> bool {
        (self.x == 0 && [0, 4, 8, 12].contains(&self.bit))
            || (self.x == Map::MAP_SIZE.x - 1 && [3, 7, 11, 15].contains(&self.bit))
            || (self.y == 0 && [0, 1, 2, 3].contains(&self.bit))
            || (self.y == Map::MAP_SIZE.y - 1 && [12, 13, 14, 15].contains(&self.bit))
    }

    pub fn get_direction(&self) -> Direction {
        match self.bit {
            0 | 5 => Direction::NorthWest,
            1 | 2 => Direction::North,
            3 | 6 => Direction::NorthEast,
            7 | 11 => Direction::East,
            10 | 12 => Direction::SouthEast,
            13 | 14 => Direction::South,
            9 | 15 => Direction::SouthWest,
            4 | 8 => Direction::West,
            _ => unreachable!(),
        }
    }

    pub fn get_closest_dig_loc(&self) -> Self {
        Loc {
            x: self.x,
            y: self.y,
            bit: match self.bit {
                0 | 5 => 1,
                3 | 6 => 2,
                9 | 12 => 13,
                10 | 15 => 14,
                b => b,
            },
        }
    }
}
