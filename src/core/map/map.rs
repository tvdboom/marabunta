use crate::core::constants::MAX_TERRAFORM_POINTS;
use crate::core::map::loc::{Direction, Loc};
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use bevy::utils::HashSet;
use pathfinding::prelude::bfs;
use rand;
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<Tile>,
}

/// The default implementation is used as starting
/// resource to draw the map seen during the menu
impl Default for Map {
    fn default() -> Self {
        Self::from_base(
            UVec2::new(Map::MAP_SIZE.x / 2 - 16, Map::MAP_SIZE.y / 2 - 6),
            0,
        )
    }
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

    // Constructors ===========================================================

    pub fn new() -> Self {
        Self {
            tiles: (0..Self::MAP_SIZE.y)
                .flat_map(|y| (0..Self::MAP_SIZE.x).map(move |x| Tile::soil(x, y)))
                .collect(),
        }
    }

    pub fn from_base(pos: UVec2, id: usize) -> Self {
        Self::new().insert_base(&pos, id).clone()
    }

    // Building methods =======================================================

    pub fn insert_base(&mut self, pos: &UVec2, id: usize) -> &mut Self {
        for (i, y) in (pos.y..pos.y + 4).enumerate() {
            for (j, x) in (pos.x..pos.x + 4).enumerate() {
                if let Some(tile) = self.tiles.iter_mut().find(|t| t.x == x && t.y == y) {
                    *tile = Tile {
                        x,
                        y,
                        texture_index: i * Map::TEXTURE_SIZE.x as usize + j,
                        base: Some(id),
                        visible: HashSet::from([id]),
                        ..default()
                    };
                }
            }
        }

        self
    }

    pub fn world(&self, id: usize) -> Vec<Tile> {
        (0..Self::WORLD_SIZE.y)
            .flat_map(|y| (0..Self::WORLD_SIZE.x).map(move |x| (x, y)))
            .map(|(x, y)| {
                if !(Self::OFFSET.x..Self::OFFSET.x + Self::MAP_SIZE.x).contains(&x)
                    || !(Self::OFFSET.y..Self::OFFSET.y + Self::MAP_SIZE.y).contains(&y)
                {
                    Tile::soil(9999, 9999)
                } else {
                    let tile = self
                        .tiles
                        .iter()
                        .find(|t| t.x == x - Self::OFFSET.x && t.y == y - Self::OFFSET.y)
                        .cloned()
                        .unwrap();

                    if tile.visible.contains(&id) {
                        tile
                    } else {
                        Tile::soil(tile.x, tile.y).with_stone(tile.has_stone)
                    }
                }
            })
            .collect::<Vec<_>>()
    }

    // Getters ================================================================

    pub fn get_coord_from_xy(x: u32, y: u32) -> Vec2 {
        Vec2::new(
            Map::MAP_VIEW.min.x + Tile::SIZE * (x as f32 + 0.5),
            Map::MAP_VIEW.max.y - Tile::SIZE * (y as f32 + 0.5),
        )
    }

    pub fn get_coord_from_loc(loc: &Loc) -> Vec2 {
        let step = 1. / (Tile::SIDE as f32 + 1.); // Steps within a tile (=0.2)
        Vec2::new(
            Self::MAP_VIEW.min.x
                + Tile::SIZE * (loc.x as f32 + (step + step * (loc.bit % Tile::SIDE) as f32)),
            Self::MAP_VIEW.max.y
                - Tile::SIZE * (loc.y as f32 + (step + step * (loc.bit / Tile::SIDE) as f32)),
        )
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        self.tiles
            .get((x % Self::MAP_SIZE.x + y * Self::MAP_SIZE.x) as usize)
    }

    fn adjacent_tile(&self, x: u32, y: u32, dir: &Direction) -> Option<usize> {
        let new_x = match dir {
            Direction::East => x + 1,
            Direction::West => x.checked_sub(1)?,
            _ => x,
        };

        let new_y = match dir {
            Direction::North => y.checked_sub(1)?,
            Direction::South => y + 1,
            _ => y,
        };

        Some((new_x % Self::MAP_SIZE.x + new_y * Self::MAP_SIZE.x) as usize)
    }

    pub fn get_adjacent_tile(&self, x: u32, y: u32, dir: &Direction) -> Option<&Tile> {
        self.adjacent_tile(x, y, dir)
            .and_then(|i| self.tiles.get(i))
    }

    pub fn get_adjacent_tile_mut(&mut self, x: u32, y: u32, dir: &Direction) -> Option<&mut Tile> {
        self.adjacent_tile(x, y, dir)
            .and_then(|i| self.tiles.get_mut(i))
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

    // Location finding =======================================================

    pub fn random_walk_loc(&self, id: usize, in_base: bool) -> Option<Loc> {
        let locations: Vec<_> = self
            .tiles
            .iter()
            .filter(|tile| tile.visible.contains(&id) && (!in_base || tile.base == Some(id)))
            .flat_map(|tile| {
                (0..16).map(move |bit| Loc {
                    x: tile.x,
                    y: tile.y,
                    bit,
                })
            })
            .filter(|loc| self.is_walkable(loc))
            .collect();

        locations.choose(&mut rand::rng()).copied()
    }

    pub fn random_dig_loc(&self, tile: Option<&Tile>, id: usize) -> Option<Loc> {
        let locations: Vec<_> = self
            .tiles
            .iter()
            .filter(|t| t.visible.contains(&id) && tile.map_or(true, |c| c.equals(t)))
            .flat_map(|t| {
                [1, 2, 7, 11, 13, 14, 4, 8].iter().map(move |&bit| Loc {
                    x: t.x,
                    y: t.y,
                    bit,
                })
            })
            .filter(|loc| {
                !self.is_walkable(loc)
                    && !loc.is_map_edge()
                    && self
                        .get_adjacent_tile(loc.x, loc.y, &loc.get_direction())
                        .map_or(false, |t| !t.has_stone)
                    && self.get_neighbors(loc).iter().any(|l| self.is_walkable(l))
            })
            .collect();

        locations.choose(&mut rand::rng()).copied()
    }

    // Pathing ================================================================

    pub fn is_walkable(&self, loc: &Loc) -> bool {
        self.get_tile(loc.x, loc.y).map_or(false, |tile| {
            tile.bitmap() & (1 << Tile::SIDE.pow(2) - loc.bit - 1) != 0
        })
    }

    pub fn get_neighbors(&self, loc: &Loc) -> Vec<Loc> {
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

        moves
            .iter()
            .filter_map(|&(dx, dy)| {
                let (mut x, mut y, mut bit) = (loc.x, loc.y, loc.bit);

                // Bit positions on the tile
                let nx = (bit % Tile::SIDE) as i8 + dx;
                let ny = (bit / Tile::SIDE) as i8 + dy;

                if nx < 0 {
                    if x == 0 {
                        return None;
                    }
                    x -= 1;
                    bit += Tile::SIDE - 1; // Move one tile left
                } else if nx >= Tile::SIDE as i8 {
                    if x + 1 >= Self::WORLD_SIZE.x {
                        return None;
                    }
                    x += 1;
                    bit -= Tile::SIDE - 1; // Move one tile right
                } else {
                    bit = (bit as i8 + dx) as u8;
                }

                if ny < 0 {
                    if y == 0 {
                        return None;
                    }
                    y -= 1;
                    bit += Tile::SIDE * (Tile::SIDE - 1); // Move one tile up
                } else if ny >= Tile::SIDE as i8 {
                    if y + 1 >= Self::WORLD_SIZE.y {
                        return None;
                    }
                    y += 1;
                    bit -= Tile::SIDE * (Tile::SIDE - 1); // Move one tile down
                } else {
                    bit = (bit as i8 + 4 * dy) as u8;
                }

                Some(Loc { x, y, bit })
            })
            .collect()
    }

    pub fn shortest_path(&self, start: &Loc, end: &Loc) -> Vec<Loc> {
        // Allow the last loc to be a wall
        bfs(
            start,
            |loc| {
                self.get_neighbors(loc)
                    .into_iter()
                    .filter(|l| self.is_walkable(l) || l == end)
                    .collect::<Vec<_>>()
            },
            |loc| loc == end,
        )
        .unwrap()
    }

    // Map updates ============================================================

    pub fn replace_tile(&mut self, tile: &Tile) {
        self.tiles[(tile.x % Self::MAP_SIZE.x + tile.y * Self::MAP_SIZE.x) as usize] = tile.clone();
    }

    /// Update the map with another map
    pub fn update(&mut self, new_map: Map) {
        self.tiles
            .iter_mut()
            .zip(new_map.tiles)
            .filter(|(_, new_t)| !new_t.is_soil())
            .for_each(|(t, new_t)| {
                *t = new_t;
            });
    }

    /// Find a tile that can replace `tile` where all directions match except those in `directions`
    pub fn find_tile(&self, tile: &Tile, directions: &HashSet<Direction>, id: usize) -> Tile {
        let mut possible_tiles = vec![];

        for texture_index in 0..Tile::MASKS.len() {
            for &rotation in &Tile::ANGLES {
                let mut tile_clone = tile.clone();
                tile_clone.visible.insert(id);

                let new_t = Tile {
                    texture_index,
                    rotation,
                    terraform: MAX_TERRAFORM_POINTS,
                    has_stone: false,
                    ..tile_clone
                };

                if Direction::iter().all(|dir| {
                    let opposite = dir.opposite();
                    if directions.contains(&opposite) {
                        new_t.border(&dir) == 0b0110
                    } else {
                        new_t.border(&dir)
                            == self
                                .get_adjacent_tile(new_t.x, new_t.y, &dir)
                                .unwrap_or(&Tile::default())
                                .border(&opposite)
                    }
                }) {
                    possible_tiles.push(new_t);
                }
            }
        }

        possible_tiles.choose(&mut rand::rng()).unwrap().clone()
    }

    pub fn find_and_replace_tile(
        &mut self,
        tile: &Tile,
        directions: &HashSet<Direction>,
        id: usize,
    ) {
        // Replace the tile
        let new_t = self.find_tile(tile, directions, id);
        self.replace_tile(&new_t);

        // Replace tiles in the provided directions
        for dir in directions.iter() {
            if let Some(t) = self.get_adjacent_tile(tile.x, tile.y, &dir.opposite()) {
                let new_t = self.find_tile(t, &HashSet::new(), id);
                self.replace_tile(&new_t);
            }
        }
    }
}
