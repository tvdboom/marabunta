use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::SpawnAntEv;
use crate::core::assets::WorldAssets;
use crate::core::camera::MainCamera;
use crate::core::constants::TILE_Z_SCORE;
use crate::core::game_settings::GameSettings;
use crate::core::map::events::SpawnTileEv;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::persistence::GameLoaded;
use crate::core::player::{Player, Players};
use bevy::prelude::*;
use rand::{rng, Rng};
use std::f32::consts::PI;

#[derive(Component)]
pub struct MapCmp;

pub fn create_map(players: &Vec<Player>) -> Map {
    let mut map = Map::default();

    if players.len() == 1 {
        // Insert base in the center of the map
        map.insert_base(
            &UVec2::new(Map::MAP_SIZE.x / 2 - 2, Map::MAP_SIZE.y / 2 - 2),
            0,
        )
        .insert_holes(10);
    } else {
        // Insert bases at random locations
        let mut bases: Vec<UVec2> = vec![];

        while bases.len() < players.len() {
            let candidate = UVec2 {
                x: rng().random_range(5..Map::MAP_SIZE.x - 9),
                y: rng().random_range(5..Map::MAP_SIZE.y - 9),
            };

            if bases
                .iter()
                .all(|pos| pos.as_vec2().distance(candidate.as_vec2()) > 10.)
            {
                bases.push(candidate);
            }
        }

        for (player, base) in players.iter().zip(bases) {
            map.insert_base(&base, player.id);
        }

        map.insert_holes(12 - 2 * players.len());
    }

    map
}

pub fn draw_map(
    mut commands: Commands,
    mut spawn_tile_ev: EventWriter<SpawnTileEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    camera: Single<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    map: Res<Map>,
    loaded: Option<Res<GameLoaded>>,
    assets: Local<WorldAssets>,
) {
    let (mut camera_t, mut projection) = camera.into_inner();

    for (i, tile) in map
        .world(&game_settings.fog_of_war, players.main_id())
        .iter()
        .enumerate()
    {
        let pos = Vec2::new(
            Map::WORLD_VIEW.min.x + Tile::SIZE * ((i as u32 % Map::WORLD_SIZE.x) as f32 + 0.5),
            Map::WORLD_VIEW.max.y - Tile::SIZE * ((i as u32 / Map::WORLD_SIZE.x) as f32 + 0.5),
        );

        spawn_tile_ev.send(SpawnTileEv {
            tile: tile.clone(),
            pos: Some(pos),
        });

        if let Some(real_tile) = map.get_tile(tile.x, tile.y) {
            if real_tile.texture_index == 9 {
                let player = players.get(*real_tile.explored.iter().next().unwrap());

                commands.spawn((
                    Sprite {
                        image: assets.image("base"),
                        custom_size: Some(Vec2::splat(Tile::SIZE + 20.)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(
                        pos.x + Tile::SIZE * 0.5,
                        pos.y - Tile::SIZE * 0.5,
                        TILE_Z_SCORE + 0.1,
                    )),
                    PickingBehavior::IGNORE,
                    MapCmp,
                ));

                // Skip spawning the queen when loading a game
                if loaded.is_none() && (player.id == players.main_id() || player.is_npc()) {
                    spawn_ant_ev.send(SpawnAntEv {
                        ant: AntCmp::new(&Ant::Queen, player),
                        transform: Transform {
                            translation: pos.extend(0.),
                            rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                            ..default()
                        },
                        entity: None,
                    });
                }

                if tile.explored.contains(&players.main_id()) {
                    // Place the camera on top of the player's base
                    projection.scale = 0.5; // Increase zoom
                    camera_t.translation = pos.extend(camera_t.translation.z);
                }

                commands.remove_resource::<GameLoaded>();
            }
        }
    }
}
