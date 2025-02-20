use crate::core::ants::components::*;
use crate::core::ants::utils::{spawn_ant, walk};
use crate::core::assets::WorldAssets;
use crate::core::constants::{ANT_Z_SCORE, EGG_Z_SCORE};
use crate::core::map::loc::Direction;
use crate::core::map::map::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use crate::core::map::utils::spawn_tile;
use crate::core::player::Player;
use crate::core::resources::GameSettings;
use crate::core::utils::scale_duration;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use std::mem::discriminant;
use strum::IntoEnumIterator;

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
                        if ant.action == Action::Die {
                            atlas.index // Remain at last frame when dead
                        } else {
                            0
                        }
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
                action: ant.action.clone(),
                timer: Timer::from_seconds(ant.action.interval(), TimerMode::Repeating),
                last_index: atlas.last_index,
            };
        }
    }
}

pub fn tile_dig(
    mut commands: Commands,
    mut ant_q: Query<&mut AntCmp>,
    mut tile_q: Query<(Entity, &Transform, &mut Tile)>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    let tile_entities: Vec<_> = tile_q.iter().map(|(e, _, t)| (e, t.x, t.y)).collect();

    for (_, tile_t, mut tile) in tile_q.iter_mut() {
        for (i, dir) in Direction::iter().enumerate() {

            // Select ants that were digging on that tile in the same direction
            let mut ants = ant_q
                .iter_mut()
                .filter(|ant| {
                    matches!(ant.action, Action::Dig(l) if l.x == tile.x && l.y == tile.y && l.direction() == dir)
                })
                .collect::<Vec<_>>();

            // Calculate the aggregate terraform progress
            let terraform = ants.len() as f32 * 20. * game_settings.speed * time.delta_secs();

            if tile.terraform[i] > terraform {
                tile.terraform[i] -= terraform;
            } else {
                // Direction is fully terraformed
                tile.terraform[i] = 100.; // Reset terraform progress for the new tile

                for new_t in map.select_new_tiles(&tile, &dir).iter_mut() {
                    commands
                        .entity(
                            tile_entities
                                .iter()
                                .find(|(_, x, y)| *x == new_t.x && *y == new_t.y)
                                .unwrap()
                                .0,
                        )
                        .despawn_recursive();

                    spawn_tile(
                        &mut commands,
                        &new_t,
                        tile_t.translation.truncate(),
                        &assets,
                    );
                }

                // Set digging ants back to idle
                ants.iter_mut().for_each(|ant| {
                    ant.action = Action::Idle;
                });
            }
        }
    }
}

pub fn hatch_eggs(
    mut commands: Commands,
    mut egg_q: Query<(Entity, &mut Egg, &Transform)>,
    game_settings: Res<GameSettings>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    for (egg_e, mut egg, egg_t) in egg_q.iter_mut() {
        egg.timer
            .tick(scale_duration(time.delta(), game_settings.speed));

        if egg.timer.just_finished() {
            spawn_ant(
                &mut commands,
                egg.ant.clone(),
                egg_t.translation.truncate(),
                &assets,
            );
            commands.entity(egg_e).despawn();
        }
    }
}

pub fn resolve_action_ants(
    mut commands: Commands,
    mut ant_q: Query<(Entity, &mut AntCmp, &mut Transform)>,
    wrapper_q: Query<(Entity, &AntHealthWrapper)>,
    mut player: ResMut<Player>,
    map: Res<Map>,
    game_settings: Res<GameSettings>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    for (ant_e, mut ant, mut ant_t) in ant_q.iter_mut() {
        match ant.action {
            Action::Die => {
                if let Some(timer) = ant.timer.as_mut() {
                    timer.tick(scale_duration(time.delta(), game_settings.speed));

                    if timer.just_finished() {
                        commands
                            .entity(wrapper_q.iter().find(|(_, w)| w.0 == ant_e).unwrap().0)
                            .despawn_recursive();
                        commands.entity(ant_e).despawn();
                    }
                }
            }
            Action::Idle => match ant.kind {
                Ant::BlackAnt => {
                    ant.action = Action::Walk(map.random_dig_loc().unwrap());
                }
                Ant::BlackQueen => {
                    if let Some(ant_c) = player.queue.first() {
                        if let Some(timer) = ant.timer.as_mut() {
                            timer.tick(scale_duration(time.delta(), game_settings.speed));

                            if timer.just_finished() {
                                commands.spawn((
                                    Sprite {
                                        image: assets.image("larva1"),
                                        ..default()
                                    },
                                    Transform {
                                        translation: ant_t
                                            .translation
                                            .truncate()
                                            .extend(EGG_Z_SCORE),
                                        rotation: ant_t.rotation,
                                        scale: Vec3::splat(0.5 * ant_c.scale),
                                        ..default()
                                    },
                                    Egg {
                                        ant: ant_c.kind.clone(),
                                        timer: Timer::from_seconds(
                                            ant_c.hatch_time,
                                            TimerMode::Once,
                                        ),
                                    },
                                    MapCmp,
                                ));

                                player.queue.remove(0);
                                ant.timer = None;
                                ant.action = Action::Walk(map.random_base_loc().unwrap());
                            }
                        } else {
                            ant.timer = Some(Timer::from_seconds(2.5, TimerMode::Once));
                        }
                    } else {
                        ant.action = Action::Walk(map.random_base_loc().unwrap());
                    }
                }
            },
            Action::Walk(target_loc) => {
                let current_loc = map.get_loc(&ant_t.translation);
                if current_loc != target_loc {
                    walk(&ant, &mut ant_t, &target_loc, &map, &game_settings, &time);
                } else {
                    // Ant reached the target loc => continue with default action
                    ant.action = match ant.kind {
                        Ant::BlackAnt => Action::Dig(current_loc),
                        Ant::BlackQueen => Action::Idle,
                    };
                }
            }
            _ => {}
        }
    }
}

pub fn update_ant_health_bars(
    ant_q: Query<
        (&Transform, &AntCmp),
        (With<AntCmp>, Without<AntHealthWrapper>, Without<AntHealth>),
    >,
    mut wrapper_q: Query<
        (Entity, &mut Transform, &mut Visibility, &AntHealthWrapper),
        (With<AntHealthWrapper>, Without<AntHealth>),
    >,
    mut health_q: Query<(&mut Transform, &mut Sprite), With<AntHealth>>,
    children_q: Query<&Children>,
) {
    for (wrapper_e, mut wrapper_t, mut wrapper_v, wrapper) in wrapper_q.iter_mut() {
        let (ant_t, ant) = ant_q.get(wrapper.0).unwrap();

        // Show the health bar when the ant is damaged
        if ant.health > 0. && ant.health < ant.max_health {
            *wrapper_v = Visibility::Visible;

            // Place the health bar on top of the ant on a distance dependent on the ant's rotation
            wrapper_t.translation = (ant_t.translation.truncate()
                + Vec2::new(
                    0.,
                    ant.size().y
                        * 0.5
                        * (ant_t.rotation.to_euler(EulerRot::ZXY).0.cos().abs() * 0.5 + 0.5),
                ))
            .extend(ANT_Z_SCORE + 0.1);

            for child in children_q.iter_descendants(wrapper_e) {
                if let Ok((mut health_t, mut health_s)) = health_q.get_mut(child) {
                    if let Some(size) = health_s.custom_size.as_mut() {
                        let full_size = ant.size().x * 0.77;
                        size.x = full_size * ant.health / ant.max_health;
                        health_t.translation.x = (size.x - full_size) * 0.5;
                    }
                }
            }
        } else {
            *wrapper_v = Visibility::Hidden;
        }
    }
}

pub fn check_keys(keyboard: Res<ButtonInput<KeyCode>>, mut player: ResMut<Player>) {
    if keyboard.just_pressed(KeyCode::KeyW) {
        player.queue.push(AntCmp::new(Ant::BlackAnt));
    }
}
