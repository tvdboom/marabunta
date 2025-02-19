use crate::core::ants::components::{Action, AnimationCmp, Ant, AntCmp};
use crate::core::ants::utils::walk;
use crate::core::assets::WorldAssets;
use crate::core::map::components::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use crate::core::resources::GameSettings;
use crate::utils::{scale_duration, NameFromEnum};
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use std::mem::discriminant;

pub fn spawn_ant(commands: &mut Commands, kind: Ant, pos: Vec2, assets: &Local<WorldAssets>) {
    let ant = AntCmp::new(kind);

    let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));
    commands.spawn((
        Sprite {
            image: atlas.image,
            texture_atlas: Some(atlas.texture),
            ..default()
        },
        Transform {
            translation: pos.extend(3. + ant.z_score),
            rotation: Quat::from_rotation_z(rand::rng().random_range(0.0..2. * PI)),
            scale: Vec3::splat(ant.scale),
            ..default()
        },
        AnimationCmp {
            timer: Timer::from_seconds(ant.action.interval(), TimerMode::Repeating),
            last_index: atlas.last_index,
            action: ant.action.clone(),
        },
        ant,
        MapCmp,
    ));
}

pub fn animate_ants(
    mut ant_q: Query<(&mut Sprite, &AntCmp, &mut AnimationCmp)>,
    game_settings: Res<GameSettings>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    for (mut sprite, ant, mut animation) in ant_q.iter_mut() {
        if discriminant(&ant.action) == discriminant(&animation.action) {
            // If the ant's action matches the animation, continue the frames
            animation
                .timer
                .tick(scale_duration(time.delta(), game_settings.speed));

            if animation.timer.just_finished() {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = if atlas.index == animation.last_index {
                        0
                    } else {
                        atlas.index + 1
                    };
                }
            }
        } else {
            // Else adjust the atlas
            let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));
            *sprite = Sprite {
                image: atlas.image,
                texture_atlas: Some(atlas.texture),
                ..default()
            };
            *animation = AnimationCmp {
                timer: Timer::from_seconds(ant.action.interval(), TimerMode::Repeating),
                last_index: atlas.last_index,
                action: ant.action.clone(),
            };
        }
    }
}

pub fn resolve_action_ants(
    mut ant_q: Query<(&mut AntCmp, &mut Transform)>,
    tile_q: Query<(Entity, &Tile)>,
    map: Res<Map>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (mut ant, mut ant_t) in ant_q.iter_mut() {
        match ant.action {
            Action::Idle => {
                match ant.kind {
                    Ant::BlackAnt => {
                        // Determine new location to dig
                        ant.action =
                            Action::Walk(map.random_dig_loc().expect("No location to dig."));
                    }
                    Ant::BlackQueen => {
                        // Determine new location to walk to
                        ant.action =
                            Action::Walk(map.random_walk_loc().expect("No location to walk."));
                    }
                }
            }
            Action::Walk(target_loc) => {
                let current_loc = Map::get_loc(&ant_t.translation);
                if current_loc != target_loc {
                    walk(&ant, &mut ant_t, &target_loc, &map, &game_settings, &time);
                } else {
                    // Ant reached the target loc => continue with default action
                    match ant.kind {
                        Ant::BlackAnt => {
                            // Rotate towards the wall
                            let neighbors = map.get_neighbors(&current_loc, false);
                            let wall = neighbors.first().unwrap();

                            let d = -ant_t.translation + Map::get_coord(wall).extend(ant_t.translation.z);
                            let angle = Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5);

                            if ant_t.rotation != angle {
                                // Rotate towards the wall
                                ant_t.rotation = ant_t.rotation.rotate_towards(
                                    angle,
                                    game_settings.speed * time.delta_secs(),
                                );
                            } else {
                                // Determine tile to dig
                                let tile_e = tile_q
                                    .iter()
                                    .find(|(_, t)| t.x == wall.x && t.y == wall.y)
                                    .map(|(e, _)| e)
                                    .expect("Current loc has no associated tile.");

                                ant.action = Action::Dig(tile_e);
                            }
                        }
                        Ant::BlackQueen => {
                            ant.action = Action::Idle;
                        }
                    }
                }
            }
            Action::Dig(entity) => {}
        }
    }
}
