use crate::core::assets::WorldAssets;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use bevy::prelude::*;

pub fn draw_start_map(mut commands: Commands, window: Query<&Window>, assets: Local<WorldAssets>) {
    let window = window.get_single().unwrap();
    let map = Map::new();

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
                Transform::from_translation(Vec3::new(
                    -window.width() * 0.5 + Tile::SIZE * x as f32,
                    window.height() * 0.5 - Tile::SIZE * y as f32,
                    0.,
                )),
                tile.clone(),
            ));
        }
    }
}
