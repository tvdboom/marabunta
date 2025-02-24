use crate::core::ants::components::Ant;
use crate::core::ants::utils::spawn_ant;
use crate::core::assets::WorldAssets;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::map::utils::spawn_tile;
use crate::core::resources::{GameMode, GameSettings};
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct MapCmp;

pub fn create_map(game_settings: &GameSettings) -> Map {
    match game_settings.mode {
        GameMode::SinglePlayer => {
            // Insert base in the center of the map
            Map::from_base(UVec2::new(Map::MAP_SIZE.x / 2 - 2, Map::MAP_SIZE.y / 2 - 2))
        }
        GameMode::MultiPlayer(n_players) => {
            let mut map = Map::new();

            // Insert bases at random locations
            let mut rng = rand::rng();
            let mut bases: Vec<UVec2> = Vec::new();

            while bases.len() < n_players {
                let candidate = UVec2 {
                    x: rng.random_range(5..Map::MAP_SIZE.x - 5),
                    y: rng.random_range(5..Map::MAP_SIZE.x - 5),
                };

                if bases
                    .iter()
                    .all(|pos| pos.as_vec2().distance(candidate.as_vec2()) >= 10.)
                {
                    bases.push(candidate);
                }
            }

            for base in bases.iter() {
                map.insert_base(base);
            }

            map
        }
    }
}

pub fn draw_map(mut commands: Commands, map: Res<Map>, assets: Local<WorldAssets>) {
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
}
