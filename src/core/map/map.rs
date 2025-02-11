use crate::core::map::tile::Tile;
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Map {
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    /// Number of tiles in the map
    pub const MAP_SIZE: UVec2 = UVec2::new(50, 28);

    /// Number of tiles to add to the map in each direction
    pub const OFFSET: UVec2 = UVec2::new(30, 15);

    /// Total world size (map + offset)
    pub const SIZE: UVec2 = UVec2::new(
        Self::MAP_SIZE.x + Self::OFFSET.x * 2,
        Self::MAP_SIZE.y + Self::OFFSET.y * 2,
    );

    /// Maximum view size
    pub const MAX_VIEW: Rect = Rect {
        min: Vec2::new(
            -(Self::SIZE.x as f32) * Tile::SIZE * 0.5,
            -(Self::SIZE.y as f32) * Tile::SIZE * 0.5,
        ),
        max: Vec2::new(
            Self::SIZE.x as f32 * Tile::SIZE * 0.5,
            Self::SIZE.y as f32 * Tile::SIZE * 0.5,
        ),
    };

    /// Maximum view size of the map (without the offset)
    pub const MAX_VIEW_MAP: Rect = Rect {
        min: Vec2::new(
            -(Self::MAP_SIZE.x as f32) * Tile::SIZE * 0.5,
            -(Self::MAP_SIZE.y as f32) * Tile::SIZE * 0.5,
        ),
        max: Vec2::new(
            Self::MAP_SIZE.x as f32 * Tile::SIZE * 0.5,
            Self::MAP_SIZE.y as f32 * Tile::SIZE * 0.5,
        ),
    };

    /// Number of tiles in the texture
    pub const TEXTURE_SIZE: UVec2 = UVec2::new(8, 9);

    /// Tiles consisting only of soil
    pub const SOIL_TILES: [usize; 3] = [29, 66, 67];

    pub fn new() -> Self {
        let angles = [0, 90, 180, 270];

        let mut rng = thread_rng();
        let tiles: Vec<Vec<Tile>> = (0..Self::SIZE.y)
            .map(|_| {
                (0..Self::SIZE.x)
                    .map(|_| {
                        // 66 is the index of the full soil tile
                        Tile::from_rotation(
                            *Self::SOIL_TILES.choose(&mut rng).unwrap(),
                            angles.choose(&mut rng).unwrap(),
                        )
                    })
                    .collect()
            })
            .collect();

        Self { tiles }
    }

    pub fn insert_base(&mut self, pos: UVec2) {
        for (i, y) in (pos.y..pos.y + 4).enumerate() {
            for (j, x) in (pos.x..pos.x + 4).enumerate() {
                self.tiles[y as usize][x as usize] =
                    Tile::new(i * Map::TEXTURE_SIZE.x as usize + j);
            }
        }

        // Add soil hole
        self.tiles[pos.y as usize + 1][pos.x as usize + 1] = Tile::new(64);
    }
}
