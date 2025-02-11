use crate::core::assets::WorldAssets;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component)]
pub struct MapCmp;

#[derive(Component)]
pub struct StoneCmp;

pub fn draw_start_map(mut commands: Commands, assets: Local<WorldAssets>) {
    let mut map = Map::new();
    map.insert_base(UVec2::new(Map::SIZE.x / 2 - 16, Map::SIZE.y / 2 - 6));

    for (y, col) in map.tiles.iter().enumerate() {
        for (x, tile) in col.iter().enumerate() {
            let texture = assets.texture("tiles");
            commands.spawn((
                Sprite {
                    image: texture.image,
                    custom_size: Some(Vec2::splat(Tile::SIZE)),
                    texture_atlas: Some(TextureAtlas {
                        layout: texture.layout,
                        index: tile.index,
                    }),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(
                        Map::MAX_VIEW.min.x + Tile::SIZE * (x as f32 + 0.5),
                        Map::MAX_VIEW.max.y - Tile::SIZE * (y as f32 + 0.5),
                        0.,
                    ),
                    rotation: Quat::from_rotation_z((tile.rotation as f32).to_radians()),
                    ..default()
                },
                tile.clone(),
                MapCmp,
            ));

            // Add random stones for decoration
            if Map::SOIL_TILES.contains(&tile.index) && rand::random::<f32>() > 0.9 {
                commands.spawn((
                    Sprite {
                        image: assets.image(&format!("stone{}", thread_rng().gen_range(1..=18))),
                        ..default()
                    },
                    Transform {
                        translation: Vec3::new(
                            Map::MAX_VIEW.min.x + Tile::SIZE * (x as f32 + 0.5),
                            Map::MAX_VIEW.max.y - Tile::SIZE * (y as f32 + 0.5),
                            1.,
                        ),
                        rotation: Quat::from_rotation_z(
                            thread_rng().gen_range(0.0_f32..360.).to_radians(),
                        ),
                        scale: Vec3::splat(thread_rng().gen_range(0.1..0.2)),
                        ..default()
                    },
                    StoneCmp,
                    MapCmp,
                ));
            }
        }
    }
}
