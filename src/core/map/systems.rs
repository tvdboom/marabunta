use crate::core::ants::components::Ant;
use crate::core::ants::utils::spawn_ant;
use crate::core::assets::WorldAssets;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::map::utils::spawn_tile;
use bevy::prelude::*;

#[derive(Component)]
pub struct MapCmp;

pub fn draw_start_map(mut commands: Commands, assets: Local<WorldAssets>) {
    let mut map = Map::new();
    map.insert_base(UVec2::new(
        Map::MAP_SIZE.x / 2 - 16,
        Map::MAP_SIZE.y / 2 - 6,
    ));

    for (i, tile) in map.world().iter_mut().enumerate() {
        let pos = Vec2::new(
            Map::WORLD_VIEW.min.x + Tile::SIZE * ((i as u32 % Map::WORLD_SIZE.x) as f32 + 0.5),
            Map::WORLD_VIEW.max.y - Tile::SIZE * ((i as u32 / Map::WORLD_SIZE.x) as f32 + 0.5),
        );

        spawn_tile(&mut commands, tile, pos, &assets);

        // Spawn queen
        if tile.texture_index == 9 {
            spawn_ant(&mut commands, Ant::BlackQueen, pos, &assets);
        }
    }

    commands.insert_resource(map);
}
