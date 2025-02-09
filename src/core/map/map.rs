use crate::core::map::tile::Tile;
use bevy::prelude::*;

pub struct Map {
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub const SIZE: UVec2 = UVec2::new(50, 50);

    pub fn new() -> Self {
        // 73 is the index of the full soil tile
        let mut tiles = vec![vec![Tile::new(66); Self::SIZE.x as usize]; Self::SIZE.y as usize];

        // Insert starting tiles
        tiles[0][0] = Tile::new(0);
        tiles[0][1] = Tile::new(1);
        tiles[0][2] = Tile::new(2);
        tiles[0][3] = Tile::new(3);
        tiles[1][0] = Tile::new(8);
        tiles[1][1] = Tile::new(9);
        tiles[1][2] = Tile::new(10);
        tiles[1][3] = Tile::new(11);
        tiles[2][0] = Tile::new(16);
        tiles[2][1] = Tile::new(17);
        tiles[2][2] = Tile::new(18);
        tiles[2][3] = Tile::new(19);
        tiles[3][0] = Tile::new(24);
        tiles[3][1] = Tile::new(25);
        tiles[3][2] = Tile::new(26);
        tiles[3][3] = Tile::new(27);

        Self { tiles }
    }
}
