use crate::core::map::tile::Tile;
use bevy::prelude::*;
use pathfinding::prelude::astar;
use rand;
use rand::prelude::IndexedRandom;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
    pub bit: u8, // 0-15, representing a position in the tile's 4x4 bit grid
}

#[derive(Resource)]
pub struct Map {
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    /// Number of tiles in the map
    pub const MAP_SIZE: UVec2 = UVec2::new(50, 28);

    /// Number of tiles to add to the map in each direction
    pub const OFFSET: UVec2 = UVec2::new(30, 15);

    /// Total world size (map + offset)
    pub const WORLD_SIZE: UVec2 = UVec2::new(
        Self::MAP_SIZE.x + Self::OFFSET.x * 2,
        Self::MAP_SIZE.y + Self::OFFSET.y * 2,
    );

    /// Size of the map (without the offset)
    pub const MAP_VIEW: Rect = Rect {
        min: Vec2::new(
            -(Self::MAP_SIZE.x as f32) * Tile::SIZE * 0.5,
            -(Self::MAP_SIZE.y as f32) * Tile::SIZE * 0.5,
        ),
        max: Vec2::new(
            Self::MAP_SIZE.x as f32 * Tile::SIZE * 0.5,
            Self::MAP_SIZE.y as f32 * Tile::SIZE * 0.5,
        ),
    };

    /// Maximum view size
    pub const WORLD_VIEW: Rect = Rect {
        min: Vec2::new(
            -(Self::WORLD_SIZE.x as f32) * Tile::SIZE * 0.5,
            -(Self::WORLD_SIZE.y as f32) * Tile::SIZE * 0.5,
        ),
        max: Vec2::new(
            Self::WORLD_SIZE.x as f32 * Tile::SIZE * 0.5,
            Self::WORLD_SIZE.y as f32 * Tile::SIZE * 0.5,
        ),
    };

    /// Number of tiles in the texture
    pub const TEXTURE_SIZE: UVec2 = UVec2::new(8, 9);

    pub fn new() -> Self {
        let tiles: Vec<Vec<Tile>> = (0..Self::MAP_SIZE.y)
            .map(|_| (0..Self::MAP_SIZE.x).map(|_| Tile::soil()).collect())
            .collect();

        Self { tiles }
    }

    pub fn world(&self) -> Vec<Vec<Tile>> {
        (0..Self::WORLD_SIZE.y)
            .map(|y| {
                (0..Self::WORLD_SIZE.x)
                    .map(|x| {
                        if y < Self::OFFSET.y
                            || y >= Self::MAP_SIZE.y + Self::OFFSET.y
                            || x < Self::OFFSET.x
                            || x >= Self::MAP_SIZE.x + Self::OFFSET.x
                        {
                            Tile::soil()
                        } else {
                            self.tiles[(y - Self::OFFSET.y) as usize][(x - Self::OFFSET.x) as usize]
                                .clone()
                        }
                    })
                    .collect()
            })
            .collect()
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

    pub fn get_coord(loc: &Loc) -> Vec2 {
        let step = 1. / (Tile::SIDE as f32 + 1.); // Steps within a tile (=0.2)
        Vec2::new(
            Self::MAP_VIEW.min.x
                + Tile::SIZE
                    * (loc.x as f32 + (step + step * (loc.bit as usize % Tile::SIDE) as f32)),
            Self::MAP_VIEW.max.y
                - Tile::SIZE
                    * (loc.y as f32 + (step + step * (loc.bit as usize / Tile::SIDE) as f32)),
        )
    }

    pub fn get_world_coord(x: usize, y: usize) -> Vec2 {
        Vec2::new(
            Self::WORLD_VIEW.min.x + Tile::SIZE * (x as f32 + 0.5),
            Self::WORLD_VIEW.max.y - Tile::SIZE * (y as f32 + 0.5),
        )
    }

    pub fn random_walkable(&self) -> Option<Loc> {
        let mut walkable_positions = vec![];

        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                for bit in 0..16 {
                    if (tile.bitmap() & (1 << bit)) != 0 {
                        walkable_positions.push(Loc { x, y, bit });
                    }
                }
            }
        }

        walkable_positions.choose(&mut rand::rng()).copied()
    }

    pub fn get_neighbors(&self, loc: Loc) -> Vec<Loc> {
        let mut neighbors = vec![];
        let (x, y, bit) = (loc.x, loc.y, loc.bit);

        // Bit positions in a 4x4 tile
        let bit_x = bit % 4;
        let bit_y = bit / 4;

        // Possible moves within the same tile
        let moves = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        for (dx, dy) in moves {
            let nx = bit_x as i8 + dx;
            let ny = bit_y as i8 + dy;

            if nx >= 0 && nx < 4 && ny >= 0 && ny < 4 {
                let new_bit = (ny * 4 + nx) as u8;
                if self.tiles[y][x].bitmap() & (1 << new_bit) != 0 {
                    neighbors.push(Loc { x, y, bit: new_bit });
                }
            }
        }

        // Moving between tiles (left, right, up, down)
        let tile_moves = [
            (-1, 0, 3, bit_y), // Left tile, bit on right edge
            (1, 0, 0, bit_y),  // Right tile, bit on left edge
            (0, -1, bit_x, 3), // Down tile, bit on top edge
            (0, 1, bit_x, 0),  // Up tile, bit on bottom edge
        ];

        for (dx, dy, new_bit_x, new_bit_y) in tile_moves {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0
                && ny >= 0
                && (ny as usize) < self.tiles.len()
                && (nx as usize) < self.tiles[0].len()
            {
                let new_bit = (new_bit_y * 4 + new_bit_x) as u8;
                if self.tiles[ny as usize][nx as usize].bitmap() & (1 << new_bit) != 0 {
                    neighbors.push(Loc {
                        x: nx as usize,
                        y: ny as usize,
                        bit: new_bit,
                    });
                }
            }
        }

        neighbors
    }

    pub fn shortest_path(&self, start: Loc, goal: Loc) -> Option<Vec<Loc>> {
        astar(
            &start,
            |loc| {
                let neighbors: Vec<_> = self.get_neighbors(*loc).iter().copied().collect();
                neighbors.into_iter().map(|loc| (loc, 1))
            },
            |loc| {
                ((loc.x as isize - goal.x as isize).abs()
                    + (loc.y as isize - goal.y as isize).abs()) as usize
            },
            |loc| *loc == goal,
        )
        .map(|(path, _)| path)
    }
}
