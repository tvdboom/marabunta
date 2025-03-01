use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::SpawnAntEv;
use crate::core::camera::{clamp_to_rect, MainCamera};
use crate::core::map::events::SpawnTileEv;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::player::Player;
use crate::core::resources::{GameMode, GameSettings};
use crate::core::states::AppState;
use bevy::prelude::*;
use rand::{rng, Rng};
use std::f32::consts::PI;

#[derive(Component)]
pub struct MapCmp;

pub fn create_map(game_settings: &GameSettings) -> Map {
    match &game_settings.mode {
        GameMode::SinglePlayer => {
            // Insert base in the center of the map
            Map::from_base(
                UVec2::new(Map::MAP_SIZE.x / 2 - 2, Map::MAP_SIZE.y / 2 - 2),
                0,
            )
        }
        GameMode::MultiPlayer(ids) => {
            let mut map = Map::new();

            // Insert bases at random locations
            let mut bases: Vec<UVec2> = Vec::new();

            while bases.len() < ids.len() {
                let candidate = UVec2 {
                    x: rng().random_range(5..Map::MAP_SIZE.x - 5),
                    y: rng().random_range(5..Map::MAP_SIZE.y - 5),
                };

                if bases
                    .iter()
                    .all(|pos| pos.as_vec2().distance(candidate.as_vec2()) == 4.)
                {
                    bases.push(candidate);
                }
            }

            for (id, base) in ids.iter().zip(bases) {
                map.insert_base(&base, *id);
            }

            map
        }
    }
}

pub fn draw_map(
    mut camera_q: Query<(&mut Transform, &OrthographicProjection), With<MainCamera>>,
    mut spawn_tile_ev: EventWriter<SpawnTileEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    map: Res<Map>,
    player: Res<Player>,
    app_state: Res<State<AppState>>,
) {
    for (i, tile) in map.world(player.id).iter_mut().enumerate() {
        let pos = Vec2::new(
            Map::WORLD_VIEW.min.x + Tile::SIZE * ((i as u32 % Map::WORLD_SIZE.x) as f32 + 0.5),
            Map::WORLD_VIEW.max.y - Tile::SIZE * ((i as u32 / Map::WORLD_SIZE.x) as f32 + 0.5),
        );

        spawn_tile_ev.send(SpawnTileEv {
            tile: tile.clone(),
            pos: Some(pos),
        });

        // Spawn queen
        if tile.texture_index == 9 {
            spawn_ant_ev.send(SpawnAntEv {
                ant: AntCmp::new(&Ant::BlackQueen, player.id),
                transform: Transform {
                    translation: pos.extend(0.),
                    rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                    ..default()
                },
            });

            // If in-game -> place camera on top of base
            if *app_state.get() == AppState::Game {
                let (mut camera_t, projection) = camera_q.single_mut();
                let view_size = projection.area.max - projection.area.min;

                // Clamp camera position within bounds
                let target_pos = clamp_to_rect(pos, view_size, Map::MAP_VIEW);
                camera_t.translation = target_pos.extend(camera_t.translation.z);
            }
        }
    }
}
