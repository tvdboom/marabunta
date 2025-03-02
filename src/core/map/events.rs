use crate::core::assets::WorldAssets;
use crate::core::constants::TILE_Z_SCORE;
use crate::core::map::map::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use crate::core::utils::{NoRotationChildCmp, NoRotationParentCmp};
use bevy::prelude::*;
use rand::{rng, Rng};
use std::f32::consts::PI;

#[derive(Event)]
pub struct SpawnTileEv {
    pub tile: Tile,
    pub pos: Option<Vec2>,
}

pub fn _spawn_tile(commands: &mut Commands, tile: &Tile, pos: Vec2, assets: &Local<WorldAssets>) {
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
            NoRotationParentCmp,
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
                    rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                    scale: Vec3::splat(rng().random_range(0.15..0.25)),
                    ..default()
                },
            ))
            .set_parent(tile_e);
    }

    if let Some(leaf) = tile.leaf.as_ref().filter(|l| l.quantity > 0.) {
        commands
            .spawn((
                Sprite {
                    image: assets.image(&leaf.image),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(0., 0., 0.2),
                    scale: Vec3::splat(leaf.quantity / 1e3),
                    ..default()
                },
                leaf.clone(),
                NoRotationChildCmp,
            ))
            .set_parent(tile_e);
    }
}

pub fn spawn_tile(
    mut commands: Commands,
    tile_q: Query<(Entity, &Tile)>,
    mut spawn_tile_ev: EventReader<SpawnTileEv>,
    assets: Local<WorldAssets>,
) {
    for SpawnTileEv { tile, pos } in spawn_tile_ev.read() {
        // Check if there already exists a tile at the same position
        if let Some((tile_e, tile_c)) = tile_q.iter().find(|(_, t)| t.x == tile.x && t.y == tile.y)
        {
            // If the tile is not soil and the texture or rotation is different, replace it
            if !tile.is_soil()
                && (tile_c.texture_index != tile.texture_index || tile_c.rotation != tile.rotation)
            {
                commands.entity(tile_e).despawn_recursive();

                _spawn_tile(
                    &mut commands,
                    &tile,
                    Map::get_coord_from_xy(tile_c.x, tile_c.y),
                    &assets,
                );
            }
        } else {
            _spawn_tile(&mut commands, &tile, pos.unwrap(), &assets);
        }
    }
}
