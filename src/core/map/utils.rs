use crate::core::assets::WorldAssets;
use crate::core::constants::TILE_Z_SCORE;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use bevy::math::{Quat, Vec2};
use bevy::prelude::*;

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

/// Spawn a tile on the map
pub fn spawn_tile(
    commands: &mut Commands,
    tile: &Tile,
    pos: Vec2,
    assets: &Local<WorldAssets>,
) -> Entity {
    let texture = assets.texture("tiles");
    commands
        .spawn((
            Sprite {
                image: texture.image,
                custom_size: Some(Vec2::splat(Tile::SIZE)),
                texture_atlas: Some(TextureAtlas {
                    layout: texture.layout,
                    index: tile.texture_index,
                }),
                ..default()
            },
            Transform {
                translation: pos.extend(TILE_Z_SCORE),
                rotation: Quat::from_rotation_z((tile.rotation as f32).to_radians()),
                ..default()
            },
            *tile,
            MapCmp,
        ))
        .id()
}
