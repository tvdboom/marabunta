use crate::core::ants::components::{Action, Ant, AntCmp};
use crate::core::ants::events::{DespawnAntEv, SpawnAntEv};
use crate::core::constants::MONSTER_SPAWN_CHANCE;
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use bevy::math::Quat;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use rand::{rng, Rng};
use std::f32::consts::PI;

pub fn spawn_enemies(
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut game_settings: ResMut<GameSettings>,
    map: Res<Map>,
) {
    map.tiles.iter().for_each(|tile| {
        if !tile.explored.is_empty() {
            if tile.texture_index == 64 && rng().random::<f32>() < MONSTER_SPAWN_CHANCE {
                spawn_ant_ev.send(SpawnAntEv {
                    ant: AntCmp::base(&Ant::Wasp),
                    transform: Transform {
                        translation: Map::get_coord_from_xy(tile.x, tile.y).extend(0.),
                        rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                        ..default()
                    },
                });
            } else if tile.texture_index == 65 && rng().random::<f32>() < MONSTER_SPAWN_CHANCE {
                // Create random termite queue
                let mut queue = vec![];
                for _ in 1..=rng().random_range(2..=10) {
                    queue.push(match rng().random::<f32>() {
                        0.5..0.6 => Ant::BlackWingedTermite,
                        0.6..0.8 => Ant::BrownTermite,
                        0.8..0.9 => Ant::BrownWingedTermite,
                        0.9..0.97 => Ant::WhiteTermite,
                        0.97..1. => Ant::WhiteWingedTermite,
                        _ => Ant::BlackTermite,
                    });
                }

                game_settings.termite_queue = HashMap::from([((tile.x, tile.y), queue)]);
            }
        }
    });

    // Spawn termites gradually from the termite queue
    for ((x, y), queue) in game_settings.termite_queue.iter_mut() {
        if rng().random::<f32>() < 0.2 {
            if let Some(ant) = queue.pop() {
                spawn_ant_ev.send(SpawnAntEv {
                    ant: AntCmp::base(&ant),
                    transform: Transform {
                        translation: Map::get_coord_from_xy(*x, *y).extend(0.),
                        rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                        ..default()
                    },
                });
            }
        }
    }
}

pub fn resolve_expeditions(
    mut ant_q: Query<(Entity, &mut Transform, &mut Visibility, &mut AntCmp)>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
) {
    for (ant_e, mut ant_t, mut ant_v, mut ant) in ant_q.iter_mut() {
        if ant.action == Action::DoNothing {
            match rng().random::<f32>() {
                0.0..0.02 => {
                    despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
                }
                0.1..0.2 if ant.kind == Ant::Worker => {
                    ant.carry += 10.;
                }
                0.2..0.25 => {
                    ant.health += 30.;
                    ant.max_health += 30.;
                    ant_t.scale *= 1.05;
                }
                0.25..0.3 => {
                    ant.damage += 5.;
                }
                0.98..1.0 => {
                    ant.action = Action::Idle;
                    *ant_v = Visibility::Inherited;
                }
                _ => (),
            }
        }
    }
}
