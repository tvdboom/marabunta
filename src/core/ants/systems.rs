use crate::core::ants::components::{Action, AnimationCmp, Ant, AntCmp};
use crate::core::ants::utils::walk;
use crate::core::assets::WorldAssets;
use crate::core::map::components::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use crate::core::resources::GameSettings;
use crate::core::utils::scale_duration;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use std::mem::discriminant;

pub fn spawn_ant(
    kind: Ant,
    pos: Vec2,
    assets: &Local<WorldAssets>,
) -> (Sprite, Transform, AnimationCmp, AntCmp, MapCmp) {
    let ant = AntCmp::new(kind);

    let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));

    (
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
    )
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

pub fn tile_dig(
    mut commands: Commands,
    mut ant_q: Query<&mut AntCmp>,
    mut tile_q: Query<(Entity, &mut Tile)>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (tile_e, mut tile) in tile_q.iter_mut() {
        let mut ants = ant_q
            .iter_mut()
            .filter(|ant| matches!(ant.action, Action::Dig(t) if t.x == tile.x && t.y == tile.y))
            .collect::<Vec<_>>();

        let terraform = ants.len() as f32 * 20. * game_settings.speed * time.delta_secs();

        if tile.terraform > terraform {
            tile.terraform -= terraform;
        } else {
            ants.iter_mut().for_each(|ant| {
                ant.action = Action::Idle;
            });
            commands.entity(tile_e).despawn();
        }
    }
}

pub fn resolve_action_ants(
    mut ant_q: Query<(&mut AntCmp, &mut Transform)>,
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
                            Action::Walk(map.random_base_loc().expect("No location to walk."));
                    }
                }
            }
            Action::Walk(target_loc) => {
                let current_loc = map.get_loc(&ant_t.translation);
                if current_loc != target_loc {
                    walk(&ant, &mut ant_t, &target_loc, &map, &game_settings, &time);
                } else {
                    // Ant reached the target loc => continue with default action
                    match ant.kind {
                        Ant::BlackAnt => {
                            ant.action = Action::Dig(*map.get_tile(&current_loc));
                        }
                        Ant::BlackQueen => {
                            ant.action = Action::Idle;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
