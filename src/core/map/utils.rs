use crate::core::constants::VISION_RANGE;
use crate::core::map::loc::Direction;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use bevy::utils::HashSet;
use strum::IntoEnumIterator;

/// Rotate a bitmap by `rotation` degrees
pub fn rotate_bitmap(bitmap: u16, rotation: i32) -> u16 {
    match rotation {
        0 => bitmap,
        90 => (0..16).fold(0, |acc, i| {
            acc | (((bitmap >> i) & 1) << ((3 - i / 4) + (i % 4) * 4))
        }),
        180 => (0..16).fold(0, |acc, i| acc | (((bitmap >> i) & 1) << (15 - i))),
        270 => (0..16).fold(0, |acc, i| {
            acc | (((bitmap >> i) & 1) << ((i / 4) + (3 - i % 4) * 4))
        }),
        _ => panic!("Invalid rotation angle"),
    }
}

/// Calculate visible tiles from a starting tile
pub fn reveal_tiles(tile: &Tile, map: &Map, depth: u32) -> HashSet<(u32, u32)> {
    let mut visible_tiles = HashSet::from([(tile.x, tile.y)]);

    // Abort search when reached the maximum vision range
    if depth < VISION_RANGE {
        for dir in Direction::iter() {
            if tile.border(&dir) != 0 {
                if let Some(tile) = map.get_adjacent_tile(tile.x, tile.y, &dir) {
                    visible_tiles.extend(reveal_tiles(tile, map, depth + 1));

                    // Also check diagonal tiles
                    if tile.border(&dir.rotate()) & 1 != 0 {
                        if let Some(tile) = map.get_adjacent_tile(tile.x, tile.y, &dir.rotate()) {
                            visible_tiles.extend(reveal_tiles(tile, map, depth + 1));
                        }
                    }
                }
            }
        }
    }

    visible_tiles
}
