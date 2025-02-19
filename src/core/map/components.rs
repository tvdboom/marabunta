use crate::core::map::tile::Tile;
use bevy::prelude::*;
use pathfinding::prelude::bfs;
use rand;
use rand::prelude::IndexedRandom;

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
}

#[derive(Resource)]
pub struct Map {
    pub tiles: Vec<Tile>,
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
        let tiles: Vec<Tile> = (0..Self::MAP_SIZE.y)
            .map(|y| {
                (0..Self::MAP_SIZE.x)
                    .map(|x| Tile::soil(x, y))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        Self { tiles }
    }

    pub fn world(&self) -> Vec<Tile> {
        (0..Self::WORLD_SIZE.y)
            .map(|y| {
                (0..Self::WORLD_SIZE.x)
                    .map(|x| {
                        if y < Self::OFFSET.y
                            || y >= Self::MAP_SIZE.y + Self::OFFSET.y
                            || x < Self::OFFSET.x
                            || x >= Self::MAP_SIZE.x + Self::OFFSET.x
                        {
                            Tile::soil(0, 0)
                        } else {
                            self.tiles
                                .iter()
                                .find(|t| {
                                    t.x == (x - Self::OFFSET.x) && t.y == (y - Self::OFFSET.y)
                                })
                                .unwrap()
                                .to_owned()
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    pub fn insert_base(&mut self, pos: UVec2) {
        for (i, y) in (pos.y..pos.y + 4).enumerate() {
            for (j, x) in (pos.x..pos.x + 4).enumerate() {
                if let Some(tile) = self.tiles.iter_mut().find(|t| t.x == x && t.y == y) {
                    let texture_index = if x == pos.x + 1 && y == pos.y + 1 {
                        64 // Add soil hole in the top left corner of the base
                    } else {
                        i * Map::TEXTURE_SIZE.x as usize + j
                    };

                    *tile = Tile {
                        x,
                        y,
                        texture_index,
                        is_base: true,
                        ..default()
                    };
                }
            }
        }
    }

    pub fn get_coord(loc: &Loc) -> Vec2 {
        let step = 1. / (Tile::SIDE as f32 + 1.); // Steps within a tile (=0.2)
        Vec2::new(
            Self::MAP_VIEW.min.x
                + Tile::SIZE * (loc.x as f32 + (step + step * (loc.bit % Tile::SIDE) as f32)),
            Self::MAP_VIEW.max.y
                - Tile::SIZE * (loc.y as f32 + (step + step * (loc.bit / Tile::SIDE) as f32)),
        )
    }

    pub fn get_tile_coord(&self, texture_index: usize) -> Vec2 {
        for tile in self.tiles.iter() {
            if tile.texture_index == texture_index {
                return Self::get_coord(&Loc {
                    x: tile.x,
                    y: tile.y,
                    bit: 5,
                });
            }
        }

        panic!("Tile not found: {texture_index}");
    }

    pub fn get_tile(&self, loc: &Loc) -> &Tile {
        &self.tiles[(loc.x % Self::MAP_SIZE.x + loc.y * Self::MAP_SIZE.x) as usize]
    }

    pub fn get_loc(&self, coord: &Vec3) -> Loc {
        let pos_x = (coord.x - Self::MAP_VIEW.min.x) / Tile::SIZE;
        let pos_y = (Self::MAP_VIEW.max.y - coord.y) / Tile::SIZE;

        let x = pos_x as u32;
        let y = pos_y as u32;

        let bit_x = (Tile::SIDE as f32 * (pos_x - x as f32)) as u8;
        let bit_y = (Tile::SIDE as f32 * (pos_y - y as f32)) as u8;

        Loc {
            x,
            y,
            bit: bit_y * Tile::SIDE + bit_x,
        }
    }

    pub fn random_base_loc(&self) -> Option<Loc> {
        let mut locations = vec![];

        for tile in self.tiles.iter().filter(|t| t.is_base) {
            for bit in 0..16 {
                let loc = Loc {
                    x: tile.x,
                    y: tile.y,
                    bit,
                };
                if self.is_walkable(&loc) {
                    locations.push(loc);
                }
            }
        }

        locations.choose(&mut rand::rng()).copied()
    }

    pub fn random_dig_loc(&self) -> Option<Loc> {
        let mut locations = vec![];

        for tile in self.tiles.iter() {
            for bit in 0..16 {
                let loc = Loc {
                    x: tile.x,
                    y: tile.y,
                    bit,
                };

                if !self.is_walkable(&loc)
                    && !loc.is_map_edge()
                    && self.get_neighbors(&loc).iter().any(|l| self.is_walkable(l))
                {
                    locations.push(loc);
                }
            }
        }

        locations.choose(&mut rand::rng()).copied()
    }

    pub fn is_walkable(&self, loc: &Loc) -> bool {
        self.get_tile(loc).bitmap() & (1 << Tile::SIDE.pow(2) - loc.bit - 1) != 0
    }

    pub fn get_neighbors(&self, loc: &Loc) -> Vec<Loc> {
        let mut neighbors = vec![];

        let moves = [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ];

        for (dx, dy) in moves {
            let (mut x, mut y, mut bit) = (loc.x, loc.y, loc.bit);

            // Bit positions on the tile
            let nx = (bit % Tile::SIDE) as i8 + dx;
            let ny = (bit / Tile::SIDE) as i8 + dy;

            (x, bit) = if nx < 0 {
                (x - 1, bit + (Tile::SIDE - 1)) // Move one tile left
            } else if nx >= Tile::SIDE as i8 {
                (x + 1, bit - (Tile::SIDE - 1)) // Move one tile right
            } else {
                (x, (bit as i8 + dx) as u8)
            };

            (y, bit) = if ny < 0 {
                (y - 1, bit + Tile::SIDE * (Tile::SIDE - 1)) // Move one tile up
            } else if ny >= Tile::SIDE as i8 {
                (y + 1, bit - Tile::SIDE * (Tile::SIDE - 1)) // Move one tile down
            } else {
                (y, (bit as i8 + 4 * dy) as u8)
            };

            neighbors.push(Loc { x, y, bit });
        }

        neighbors
    }

    pub fn shortest_path(&self, start: &Loc, goal: &Loc) -> Vec<Loc> {
        // Allow the last loc to be a wall
        bfs(
            start,
            |loc| {
                self.get_neighbors(loc)
                    .into_iter()
                    .filter(|l| self.is_walkable(l) || l == goal)
                    .collect::<Vec<_>>()
            },
            |loc| loc == goal,
        )
        .expect("No path found.")
    }
}
