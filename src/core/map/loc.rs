use crate::core::map::map::Map;
use std::f32::consts::PI;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    pub fn degrees(&self) -> f32 {
        match self {
            Direction::North => 0.,
            Direction::East => -PI * 0.5,
            Direction::South => PI,
            Direction::West => PI * 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
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
            1 | 2 => Direction::North,
            7 | 11 => Direction::East,
            13 | 14 => Direction::South,
            4 | 8 => Direction::West,
            _ => unreachable!(),
        }
    }
}
