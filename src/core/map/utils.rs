use crate::core::assets::WorldAssets;
use crate::core::constants::TILE_Z_SCORE;
use crate::core::map::map::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use bevy::math::{Quat, Vec2};
use bevy::prelude::*;
use rand::{rng, Rng};

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
pub fn spawn_tile(commands: &mut Commands, tile: &Tile, pos: Vec2, assets: &Local<WorldAssets>) {
    let texture = assets.texture("tiles");
    let tile_e = commands
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
                rotation: Quat::from_rotation_z((-tile.rotation as f32).to_radians()),
                ..default()
            },
            tile.clone(),
            MapCmp,
        ))
        .id();

    if tile.has_stone {
        commands
            .spawn((
                Sprite {
                    image: assets.image(&format!("stone{}", rng().random_range(1..=18))),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(0., 0., 0.1),
                    rotation: Quat::from_rotation_z(rng().random_range(0.0_f32..360.).to_radians()),
                    scale: Vec3::splat(rng().random_range(0.15..0.25)),
                    ..default()
                },
            ))
            .set_parent(tile_e);
    }
}

/// Replace a tile with a new one
pub fn replace_tile(
    commands: &mut Commands,
    tile: &Tile,
    tile_q: &Query<(Entity, &Tile)>,
    assets: &Local<WorldAssets>,
) {
    let (tile_e, tile_c) = tile_q
        .iter()
        .find(|(_, t)| t.x == tile.x && t.y == tile.y)
        .unwrap();

    if tile_c.texture_index != tile.texture_index || tile_c.rotation != tile.rotation {
        commands.entity(tile_e).try_despawn_recursive();

        spawn_tile(
            commands,
            &tile,
            Map::get_coord_from_xy(tile.x, tile.y),
            &assets,
        );
    }
}
