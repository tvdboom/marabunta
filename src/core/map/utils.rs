use bevy::utils::HashSet;
use strum::IntoEnumIterator;
use crate::core::constants::VISION_RANGE;
use crate::core::map::loc::Direction;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;

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

/// Add visible tiles from a starting tile to the `visible_tiles` set
pub fn reveal_surrounding_tiles(tile: &Tile, map: &Map) -> HashSet<(u32, u32)> {
    let mut visible_tiles = HashSet::from((tile.x, tile.y));

    for dir in Direction::iter() {
        if let Some(tile) = map.get_adjacent_tile(tile.x, tile.y, &dir) {
            visible_tiles.insert((tile.x, tile.y));
            reveal_surrounding_tiles(&tile, map, &mut visible_tiles, &dir);
        }
        visible_tiles.extend(reveal_surrounding_tiles(, &dir, map));

        // Check diagonal tiles
        if let Some(diag_tile) = map.get_adjacent_tile(tile.x, tile.y, &dir.rotate()) {
            reveal_surrounding_tiles(&map, &mut visible_tiles, &diag_tile, &dir);
        }
    }


    let mut current_tile = Some(start_tile.clone());

    for _ in 0..VISION_RANGE {
        if let Some(tile) = current_tile {
            if tile.border(dir) == 0 {
                break; // Stop if there's a border blocking vision
            }

            if let Some(next_tile) = map.get_adjacent_tile(tile.x, tile.y, dir) {
                visible_tiles.insert((next_tile.x, next_tile.y));
                current_tile = Some(next_tile.clone());
            } else {
                break;
            }
        }
    }
}
