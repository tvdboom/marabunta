use crate::core::constants::{MAX_TERRAFORM_POINTS, TILE_LEAF_CHANCE};
use crate::core::map::loc::{Direction, Loc};
use crate::core::map::tile::{Leaf, Tile};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::utils::HashSet;
use bevy_renet::renet::ClientId;
use pathfinding::prelude::astar;
use rand;
use rand::prelude::{IndexedRandom, IteratorRandom};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize)]
pub struct PathCache {
    /// Cache the path between two tiles
    pub paths: HashMap<((u32, u32), (u32, u32)), Vec<Loc>>,

    /// Cache the tiles that contain a location
    /// This is used for efficient removal of cache entries
    pub nodes: HashMap<(u32, u32), Vec<((u32, u32), (u32, u32))>>,
}

impl PathCache {
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn get(&self, start: &Loc, end: &Loc) -> Option<&Vec<Loc>> {
        self.paths.get(&((start.x, start.y), (end.x, end.y)))
    }

    pub fn contains_key(&self, start: &Loc, end: &Loc) -> bool {
        self.paths
            .contains_key(&((start.x, start.y), (end.x, end.y)))
    }

    pub fn insert(&mut self, start: Loc, end: Loc, path: Vec<Loc>) {
        let key = ((start.x, start.y), (end.x, end.y));
        for loc in path.iter() {
            self.nodes
                .entry((loc.x, loc.y))
                .or_insert_with(Vec::new)
                .push(key);
        }
        self.paths.insert(key, path);
    }

    pub fn invalidate(&mut self, tile: &Tile) {
        if let Some(keys) = self.nodes.get(&(tile.x, tile.y)) {
            for key in keys.iter() {
                self.paths.remove(key);
            }
        }
    }

    pub fn update(&mut self, other: PathCache) {
        self.paths.extend(other.paths);
        self.nodes.extend(other.nodes);
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub cache: PathCache,
}

/// The default implementation is used as starting
/// resource to draw the map seen during the menu
impl Default for Map {
    fn default() -> Self {
        Self {
            tiles: (0..Self::MAP_SIZE.y)
                .flat_map(|y| (0..Self::MAP_SIZE.x).map(move |x| Tile::soil(x, y)))
                .collect(),
            cache: PathCache::new(),
        }
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

    pub fn from_base(pos: UVec2, id: ClientId) -> Self {
        Self::default().insert_base(&pos, id).clone()
    }

    // Building methods =======================================================

    pub fn insert_holes(&mut self, n: usize) -> &mut Self {
        // Insert holes at random locations
        let mut holes: Vec<UVec2> = Vec::new();

        let base_positions: Vec<UVec2> = self
            .tiles
            .iter()
            .filter_map(|t| (!t.is_soil()).then_some(UVec2::new(t.x, t.y)))
            .collect();

        while holes.len() < n {
            let candidate = UVec2 {
                x: rng().random_range(0..Map::MAP_SIZE.x - 3),
                y: rng().random_range(0..Map::MAP_SIZE.y - 3),
            };

            if holes
                .iter()
                .chain(base_positions.iter())
                .all(|pos| pos.as_vec2().distance(candidate.as_vec2()) > 4.)
            {
                holes.push(candidate);
            }
        }

        for pos in holes.iter() {
            for (y, i) in (pos.y..pos.y + 3).zip([0, 1, 3]) {
                for (x, j) in (pos.x..pos.x + 3).zip([0, 1, 3]) {
                    if let Some(tile) = self.tiles.iter_mut().find(|t| t.x == x && t.y == y) {
                        *tile = Tile {
                            x,
                            y,
                            texture_index: if i == 1 && j == 1 {
                                *[64, 65].choose(&mut rng()).unwrap()
                            } else {
                                i * Map::TEXTURE_SIZE.x as usize + j
                            },
                            ..default()
                        };
                    }
                }
            }
        }

        self
    }

    pub fn insert_base(&mut self, pos: &UVec2, id: ClientId) -> &mut Self {
        for (i, y) in (pos.y..pos.y + 4).enumerate() {
            for (j, x) in (pos.x..pos.x + 4).enumerate() {
                if let Some(tile) = self.tiles.iter_mut().find(|t| t.x == x && t.y == y) {
                    let texture_index = i * Map::TEXTURE_SIZE.x as usize + j;
                    *tile = Tile {
                        x,
                        y,
                        texture_index,
                        base: if i > 0 && i < 3 && j > 0 && j < 3 {
                            Some(id)
                        } else {
                            None
                        },
                        leaf: if texture_index == 26 {
                            Some(Leaf::default())
                        } else {
                            None
                        },
                        visible: HashSet::from([id]),
                        ..default()
                    };
                }
            }
        }

        self
    }

    pub fn world(&self, id: ClientId) -> Vec<Tile> {
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

    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        self.tiles
            .get_mut((x % Self::MAP_SIZE.x + y * Self::MAP_SIZE.x) as usize)
    }

    pub fn get_tile_from_coord(&self, coord: &Vec3) -> Option<&Tile> {
        let loc = self.get_loc(coord);
        self.get_tile(loc.x, loc.y)
    }

    pub fn get_tile_mut_from_coord(&mut self, coord: &Vec3) -> Option<&mut Tile> {
        let loc = self.get_loc(coord);
        self.get_tile_mut(loc.x, loc.y)
    }

    fn adjacent_tile(&self, x: u32, y: u32, dir: &Direction) -> Option<usize> {
        let new_x = match dir {
            Direction::East | Direction::NorthEast | Direction::SouthEast => x + 1,
            Direction::West | Direction::NorthWest | Direction::SouthWest => x.checked_sub(1)?,
            _ => x,
        };

        let new_y = match dir {
            Direction::North | Direction::NorthEast | Direction::NorthWest => y.checked_sub(1)?,
            Direction::South | Direction::SouthEast | Direction::SouthWest => y + 1,
            _ => y,
        };

        Some((new_x % Self::MAP_SIZE.x + new_y * Self::MAP_SIZE.x) as usize)
    }

    pub fn get_adjacent_tile(&self, x: u32, y: u32, dir: &Direction) -> Option<&Tile> {
        self.adjacent_tile(x, y, dir)
            .and_then(|i| self.tiles.get(i))
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

    pub fn random_loc(&self, id: ClientId, in_base: bool) -> Option<Loc> {
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

        locations.choose(&mut rng()).copied()
    }

    pub fn random_loc_max_distance(&mut self, id: ClientId, loc: &Loc, d: usize) -> Option<Loc> {
        let locations: Vec<_> = self
            .tiles
            .iter()
            .filter(|tile| tile.visible.contains(&id))
            .flat_map(|tile| {
                (0..16).map(move |bit| Loc {
                    x: tile.x,
                    y: tile.y,
                    bit,
                })
            })
            .filter(|l| self.is_walkable(l))
            .collect();

        locations
            .iter()
            .filter(|l| self.shortest_path(loc, l).len() <= d)
            .choose(&mut rng())
            .copied()
    }

    pub fn random_leaf_loc(&self, id: ClientId) -> Option<Loc> {
        let locations: Vec<_> = self
            .tiles
            .iter()
            .filter(|t| t.visible.contains(&id) && t.leaf.is_some())
            .map(|t| Loc {
                x: t.x,
                y: t.y,
                bit: *[5, 6, 9, 10].choose(&mut rng()).unwrap(),
            })
            .collect();

        locations.choose(&mut rng()).copied()
    }

    pub fn closest_leaf_loc(&self, pos: &Vec3, id: ClientId) -> Option<Loc> {
        self.tiles
            .iter()
            .filter(|t| t.visible.contains(&id) && t.leaf.is_some())
            .map(|t| Loc {
                x: t.x,
                y: t.y,
                bit: *[5, 6, 9, 10].choose(&mut rng()).unwrap(),
            })
            .min_by_key(|t| {
                Map::get_coord_from_xy(t.x, t.y)
                    .extend(pos.z)
                    .distance(*pos) as u32
            })
    }

    pub fn random_dig_loc(&self, tile: Option<&Tile>, id: ClientId) -> Option<Loc> {
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

        locations.choose(&mut rng()).copied()
    }

    pub fn random_enemy_loc(&self, id: ClientId) -> Option<Loc> {
        let locations: Vec<_> = self
            .tiles
            .iter()
            .filter(|tile| tile.visible.contains(&id) && tile.visible.len() > 1)
            .flat_map(|tile| {
                (0..16).map(move |bit| Loc {
                    x: tile.x,
                    y: tile.y,
                    bit,
                })
            })
            .filter(|loc| self.is_walkable(loc))
            .collect();

        locations.choose(&mut rng()).copied()
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

    /// Use A* to find the shortest path between two locations
    fn find_path(&self, start: &Loc, end: &Loc) -> Vec<Loc> {
        let p = astar(
            start,
            |loc| {
                self.get_neighbors(loc)
                    .into_iter()
                    .filter(|l| self.is_walkable(l) || l == end) // Allow the last loc to be a wall
                    .map(|n| (n, 1))
                    .collect::<Vec<_>>()
            },
            |loc| 4 * (start.x as i32 - start.y as i32).abs() - (loc.x as i32 - loc.y as i32).abs(),
            |loc| loc == end,
        )
        .map(|(path, _)| path);

        if let Some(p) = p {
            p
        } else {
            thread::sleep(Duration::new(1000, 0)); // Waits for 10 seconds
            panic!("No path found from {:?} to {:?}.", start, end);
        }
        // .expect(format!("No path found from {:?} to {:?}.", start, end).as_str())
    }

    /// Find the shortest path between two locations (using the cache if available)
    pub fn shortest_path(&mut self, start: &Loc, end: &Loc) -> Vec<Loc> {
        // If within 2 tiles range, calculate the path directly
        if (start.x as i32 - end.x as i32).abs() + (start.y as i32 - end.y as i32).abs() <= 2 {
            return self.find_path(start, end);
        }

        // Store the calculated path in the cache if not available
        if !self.cache.contains_key(start, end) {
            let path = self.find_path(start, end);
            self.cache.insert(*start, *end, path.to_vec().clone());
            self.cache
                .insert(*end, *start, path.iter().rev().cloned().collect::<Vec<_>>());
        }

        // Use only the cached path excluding the first and last tile
        let middle_tiles: Vec<Loc> = self
            .cache
            .get(start, end)
            .unwrap()
            .iter()
            .skip_while(|l| (l.x == start.x && l.y == start.y) || (l.x == end.x && l.y == end.y))
            .cloned()
            .collect();

        // Calculate a new path for the first and last tile only
        let mut first_tile = self.find_path(start, middle_tiles.first().unwrap());
        first_tile.pop();
        let last_tile = self
            .find_path(middle_tiles.last().unwrap(), end)
            .split_off(1);

        first_tile
            .into_iter()
            .chain(middle_tiles.into_iter())
            .chain(last_tile.into_iter())
            .collect()
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
            .filter(|(t, new_t)| {
                !new_t.is_soil() && new_t.bitmap().count_ones() >= t.bitmap().count_ones()
            })
            .for_each(|(t, new_t)| {
                *t = new_t;
            });

        self.cache.update(new_map.cache);
    }

    /// Find a tile that can replace `tile` where all directions match except those in `directions`
    pub fn find_tile(&self, tile: &Tile, directions: &HashSet<Direction>, id: ClientId) -> Tile {
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

                if Direction::CARDINALS.iter().all(|dir| {
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

        possible_tiles.choose(&mut rng()).unwrap().clone()
    }

    pub fn find_and_replace_tile(
        &mut self,
        tile: &Tile,
        directions: &HashSet<Direction>,
        id: ClientId,
    ) {
        // Replace the tile dug
        let mut new_t = self.find_tile(tile, directions, id);

        // Add (possibly) a leaf on newly dug tiles
        if new_t.leaf.is_none()
            && !tile.visible.contains(&id)
            && rng().random::<f32>() < TILE_LEAF_CHANCE
        {
            new_t.leaf = Some(Leaf::new())
        }

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
