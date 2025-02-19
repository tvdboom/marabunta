use crate::core::ants::components::{Action, AnimationCmp, Ant, AntCmp, AntHealth, Egg};
use crate::core::ants::utils::{spawn_ant, walk};
use crate::core::assets::WorldAssets;
use crate::core::constants::{EGG_Z_SCORE, GAME_SPEED_STEP, MAX_GAME_SPEED};
use crate::core::map::components::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use crate::core::player::Player;
use crate::core::resources::GameSettings;
use crate::core::states::PauseState;
use crate::core::utils::scale_duration;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use std::mem::discriminant;

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
    mut ant_q: Query<(&mut AntCmp, &mut Transform)>,
    mut player: ResMut<Player>,
    map: Res<Map>,
    game_settings: Res<GameSettings>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    for (mut ant, mut ant_t) in ant_q.iter_mut() {
        match ant.action {
            Action::Idle => match ant.kind {
                Ant::BlackAnt => {
                    ant.action = Action::Walk(map.random_dig_loc().unwrap());
                }
                Ant::BlackQueen => {
                    if let Some(ant_c) = player.queue.first() {
                        if let Some(timer) = ant.brooding_timer.as_mut() {
                            timer.tick(scale_duration(time.delta(), game_settings.speed));
                            if timer.just_finished() {
                                commands.spawn((
                                    Sprite {
                                        image: assets.image("egg"),
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
                                        timer: Timer::from_seconds(ant_c.brooding, TimerMode::Once),
                                    },
                                    MapCmp,
                                ));

                                player.queue.remove(0);
                                ant.brooding_timer = None;
                                ant.action = Action::Walk(map.random_base_loc().unwrap());
                            }
                        } else {
                            ant.brooding_timer = Some(Timer::from_seconds(2.5, TimerMode::Once));
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
                        Ant::BlackAnt => Action::Dig(*map.get_tile(&current_loc)),
                        Ant::BlackQueen => Action::Idle,
                    };
                }
            }
            _ => {}
        }
    }
}

pub fn update_ant_health_bars(
    mut ant_q: Query<(Entity, &mut AntCmp)>,
    children_q: Query<&Children>,
    mut health_q: Query<(&mut Transform, &mut Sprite), With<AntHealth>>,
) {
    for (ant_e, mut ant) in ant_q.iter_mut() {
        if ant.health < ant.max_health {
            if ant.health == 0. {
                ant.action = Action::Die;

            } else {
                for child in children_q.iter_descendants(ant_e) {
                    if let Ok((mut sprite_t, mut sprite)) = health_q.get_mut(child) {
                        if let Some(size) = sprite.custom_size.as_mut() {
                            let full_size = enemy.dim.x * 0.8 - 2.0;
                            size.x = full_size * enemy.health / enemy.max_health;
                            sprite_t.translation.x = (size.x - full_size) * 0.5;
                        }
                    }
                }
            }
        }
    }
}

pub fn check_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: ResMut<Player>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut game_settings: ResMut<GameSettings>,
) {
    if keyboard.just_pressed(KeyCode::KeyW) {
        player.queue.push(AntCmp::new(Ant::BlackAnt));
    }

    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard.just_pressed(KeyCode::ArrowLeft) && game_settings.speed >= GAME_SPEED_STEP {
            game_settings.speed -= GAME_SPEED_STEP;
            if game_settings.speed == 0. {
                next_pause_state.set(PauseState::Paused);
            }
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) && game_settings.speed <= MAX_GAME_SPEED {
            game_settings.speed += GAME_SPEED_STEP;
            if game_settings.speed == GAME_SPEED_STEP {
                next_pause_state.set(PauseState::Running);
            }
        }
    }
}
