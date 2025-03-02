use crate::core::ants::components::*;
use crate::core::ants::events::{DespawnAntEv, SpawnAntEv};
use crate::core::ants::utils::walk;
use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::map::events::SpawnTileEv;
use crate::core::map::loc::Direction;
use crate::core::map::map::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use crate::core::player::Player;
use crate::core::resources::{GameSettings, Population};
use crate::core::utils::scale_duration;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::utils::HashSet;
use rand::{rng, Rng};
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
            let atlas = assets.atlas(&format!(
                "{}_{}",
                ant.kind.to_snake(),
                ant.action.animation().to_snake()
            ));

            *sprite = Sprite {
                image: atlas.image,
                texture_atlas: Some(atlas.texture),
                ..default()
            };

            *animation = AnimationCmp {
                action: ant.action.clone(),
                timer: Timer::from_seconds(ant.action.animation().interval(), TimerMode::Repeating),
                last_index: atlas.last_index,
            };
        }
    }
}

pub fn resolve_harvesting(
    mut ant_q: Query<(&Transform, &mut AntCmp)>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
) {
    for (ant_t, mut ant) in ant_q
        .iter_mut()
        .filter(|(_, a)| a.owner == player.id && a.action == Action::Harvest)
    {
        if let Some(leaf) = map
            .get_tile_mut_from_coord(&ant_t.translation)
            .unwrap()
            .leaf
            .as_mut()
            .filter(|l| l.quantity > 0.)
        {
            let carry =
                (HARVEST_SPEED * game_settings.speed * time.delta_secs()).min(leaf.quantity);

            if ant.carry + carry > ant.max_carry {
                ant.carry = ant.max_carry;
                leaf.quantity -= ant.max_carry - ant.carry;
                ant.action = Action::Idle;
            } else {
                ant.carry += carry;
                leaf.quantity -= carry;
            }
        } else {
            ant.action = Action::Idle;
        }
    }
}

pub fn resolve_digging(
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    mut tile_q: Query<&mut Tile>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
) {
    for mut tile in tile_q.iter_mut() {
        // Select ants that were digging on that tile
        let mut ants: Vec<_> = ant_q
            .iter_mut()
            .filter(|(_, ant)| {
                ant.owner == player.id && matches!(&ant.action, Action::Dig(t) if t.equals(&tile))
            })
            .collect();

        // Turn ants towards the direction they are digging
        let mut directions = HashSet::new();
        ants.iter_mut().for_each(|(t, _)| {
            let d = map.get_loc(&t.translation).get_direction();
            t.rotation = t.rotation.rotate_towards(
                Quat::from_rotation_z(d.degrees()),
                2. * game_settings.speed * time.delta_secs(),
            );
            directions.insert(d);
        });

        // Calculate the aggregate terraform progress
        let terraform = ants.len() as f32 * DIG_SPEED * game_settings.speed * time.delta_secs();

        if tile.terraform > terraform {
            tile.terraform -= terraform;
        } else {
            map.find_and_replace_tile(&tile, &directions, player.id);

            // Set digging ants onto a new task
            ants.iter_mut().for_each(|(_, ant)| {
                if rng().random::<f32>() < SAME_TUNNEL_DIG_CHANCE {
                    ant.action = Action::Walk(
                        map.random_dig_loc(Some(&tile), player.id)
                            // If there are no digging locations on the tile, select a random one
                            .unwrap_or(map.random_dig_loc(None, player.id).unwrap()),
                    );
                } else {
                    ant.action = Action::Idle;
                }
            });
        }
    }
}

pub fn hatch_eggs(
    mut commands: Commands,
    mut egg_q: Query<(Entity, &mut Egg, &Transform)>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    game_settings: Res<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
) {
    for (egg_e, mut egg, egg_t) in egg_q
        .iter_mut()
        .filter(|(_, egg, _)| egg.owner == player.id)
    {
        egg.timer
            .tick(scale_duration(time.delta(), game_settings.speed));

        if egg.timer.just_finished() {
            spawn_ant_ev.send(SpawnAntEv {
                ant: AntCmp::new(&egg.ant, player.id),
                transform: egg_t.clone(),
            });
            commands.entity(egg_e).despawn();
        }
    }
}

pub fn resolve_action_ants(
    mut commands: Commands,
    mut ant_q: Query<(Entity, &mut Transform, &mut AntCmp)>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
    map: Res<Map>,
    game_settings: Res<GameSettings>,
    mut player: ResMut<Player>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    let id = player.id;
    let cloned_query = ant_q
        .iter()
        .map(|(_, t, a)| (t.clone(), a.clone()))
        .collect::<Vec<_>>();

    for (ant_e, mut ant_t, mut ant) in ant_q.iter_mut().filter(|(_, _, ant)| ant.owner == id) {
        match ant.action {
            Action::Die => {
                if let Some(timer) = ant.timer.as_mut() {
                    timer.tick(scale_duration(time.delta(), game_settings.speed));

                    if timer.just_finished() {
                        despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
                    }
                }
            }
            Action::Idle => match ant.behavior {
                Behavior::Attack => {
                    if let Some(loc) = map.random_enemy_walk_loc(player.id) {
                        ant.action = Action::Walk(loc);
                    } else {
                        ant.action = Action::Walk(map.random_walk_loc(player.id, false).unwrap());
                    }
                }
                Behavior::Brood => {
                    if let Some(ant_queue) = player.queue.first() {
                        let ant_c = AntCmp::new(ant_queue, player.id);

                        if let Some(timer) = ant.timer.as_mut() {
                            timer.tick(scale_duration(time.delta(), game_settings.speed));

                            if timer.just_finished() {
                                commands.spawn((
                                    Sprite {
                                        image: assets.image("larva2"),
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
                                        owner: player.id,
                                        timer: Timer::from_seconds(
                                            ant_c.hatch_time,
                                            TimerMode::Once,
                                        ),
                                    },
                                    MapCmp,
                                ));

                                player.queue.remove(0);
                                ant.timer = None;
                                ant.action =
                                    Action::Walk(map.random_walk_loc(player.id, true).unwrap());
                            }
                        } else {
                            ant.timer = Some(Timer::from_seconds(BROODING_TIME, TimerMode::Once));
                        }
                    } else {
                        // If nothing in the queue, wander around through the base
                        ant.action = Action::Walk(map.random_walk_loc(player.id, true).unwrap());
                    }
                }
                Behavior::Dig => {
                    ant.action = Action::Walk(
                        map.random_dig_loc(None, player.id)
                            .unwrap_or(map.random_walk_loc(player.id, false).unwrap()),
                    );
                }
                Behavior::Harvest => {
                    if ant.carry == 0. {
                        ant.action = Action::Walk(
                            map.random_leaf_loc(player.id)
                                .unwrap_or(map.random_walk_loc(player.id, false).unwrap()),
                        );
                    } else {
                        // If it's carrying something, return to the queen
                        ant.action = Action::TargetedWalk(
                            cloned_query
                                .iter()
                                .filter_map(|(_, a)| {
                                    if a.owner == player.id && a.kind == Ant::BlackQueen {
                                        Some(a.id)
                                    } else {
                                        None
                                    }
                                })
                                .last()
                                .unwrap(),
                        )
                    }
                }
                Behavior::Wander => {
                    ant.action = Action::Walk(map.random_walk_loc(player.id, false).unwrap());
                }
            },
            Action::TargetedWalk(id) => {
                if let Some((target_t, _)) = cloned_query.iter().find(|(_, a)| a.id == id) {
                    let current_loc = map.get_loc(&ant_t.translation);
                    let target_loc = map.get_loc(&target_t.translation);

                    if current_loc != target_loc {
                        walk(&ant, &mut ant_t, &target_loc, &map, &game_settings, &time);
                    } else {
                        // Ant reached the target loc => continue with default action
                        ant.action = match ant.behavior {
                            Behavior::Harvest => {
                                // Ant reached the queen, leave food and go back to harvesting
                                player.food += ant.carry;
                                ant.carry = 0.;
                                Action::Idle
                            }
                            _ => unreachable!(),
                        }
                    }
                } else {
                    // The target doesn't exist anymore, go back to default behavior
                    ant.action = Action::Idle;
                }
            }
            Action::Walk(target_loc) => {
                let current_loc = map.get_loc(&ant_t.translation);
                if current_loc != target_loc {
                    walk(&ant, &mut ant_t, &target_loc, &map, &game_settings, &time);
                } else {
                    // Ant reached the target loc => continue with default action
                    ant.action = match ant.behavior {
                        Behavior::Dig => {
                            if map.is_walkable(&current_loc) {
                                // The tile could have been dug while it was getting there
                                Action::Idle
                            } else {
                                Action::Dig(
                                    map.get_adjacent_tile(
                                        current_loc.x,
                                        current_loc.y,
                                        &current_loc.get_direction(),
                                    )
                                    .unwrap()
                                    .clone(),
                                )
                            }
                        }
                        Behavior::Harvest => {
                            if map
                                .get_tile(current_loc.x, current_loc.y)
                                .unwrap()
                                .leaf
                                .is_some()
                            {
                                // The leaf could have been harvested completely while getting there
                                Action::Harvest
                            } else {
                                Action::Idle
                            }
                        }
                        _ => Action::Idle,
                    };
                }
            }
            _ => {}
        }
    }
}

pub fn update_ant_components(
    ant_q: Query<
        (Entity, &Transform, &AntCmp),
        (
            With<AntCmp>,
            Without<AntHealthWrapperCmp>,
            Without<AntHealthCmp>,
        ),
    >,
    mut wrapper_q: Query<
        (Entity, &mut Transform, &mut Visibility),
        (With<AntHealthWrapperCmp>, Without<AntHealthCmp>),
    >,
    mut health_q: Query<(&mut Transform, &mut Sprite), With<AntHealthCmp>>,
    mut leaf_q: Query<&mut Visibility, (With<LeafCarryCmp>, Without<AntHealthWrapperCmp>)>,
    children_q: Query<&Children>,
) {
    for (ant_e, ant_t, ant) in ant_q.iter() {
        for child in children_q.iter_descendants(ant_e) {
            if let Ok(mut leaf_v) = leaf_q.get_mut(child) {
                *leaf_v = if ant.carry > 0. {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }

            if let Ok((wrapper_e, mut wrapper_t, mut wrapper_v)) = wrapper_q.get_mut(child) {
                // Show the health bar when the ant is damaged
                if ant.health > 0. && ant.health < ant.max_health {
                    *wrapper_v = Visibility::Visible;

                    // Place the health bar on top of the ant on a distance dependent on the ant's rotation
                    wrapper_t.translation = Vec3::new(
                        ant.size().x * 0.5 * ant_t.rotation.to_euler(EulerRot::ZXY).0.sin(),
                        ant.size().y * 0.5 * ant_t.rotation.to_euler(EulerRot::ZXY).0.cos(),
                        0.1,
                    );

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
    }
}

pub fn update_vision(
    mut ant_q: Query<(Entity, &mut Transform, &AntCmp)>,
    mut spawn_tile_ev: EventWriter<SpawnTileEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
    player: Res<Player>,
    mut map: ResMut<Map>,
    population: Res<Population>,
) {
    let mut visible_tiles = HashSet::new();

    ant_q
        .iter()
        .filter(|(_, _, a)| a.owner == player.id)
        .for_each(|(_, ant_t, _)| {
            let tile = map.get_tile_from_coord(&ant_t.translation).unwrap();
            visible_tiles.insert((tile.x, tile.y));

            for dir in Direction::iter() {
                if tile.border(&dir) != 0 {
                    if let Some(tile) = map.get_adjacent_tile(tile.x, tile.y, &dir) {
                        visible_tiles.insert((tile.x, tile.y));

                        // Also update tiles in corners (north-west, south-east, etc...)
                        // if the corner bit is walkable
                        if tile.border(&dir.rotate()) & 1 != 0 {
                            if let Some(tile) = map.get_adjacent_tile(tile.x, tile.y, &dir.rotate())
                            {
                                visible_tiles.insert((tile.x, tile.y));
                            }
                        }
                    }
                }
            }
        });

    visible_tiles.iter().for_each(|(x, y)| {
        let tile = map.get_tile_mut(*x, *y).unwrap();

        tile.visible.insert(player.id);
        spawn_tile_ev.send(SpawnTileEv {
            tile: tile.clone(),
            pos: None,
        });
    });

    // Show/hide enemies on the map
    let mut current_population = vec![];
    for (ant_e, mut ant_t, ant) in ant_q.iter_mut().filter(|(_, _, a)| a.owner != player.id) {
        current_population.push(ant.id);
        if let Some((t, _)) = population.0.values().find(|(_, a)| a.id == ant.id) {
            // The ant is already on the map
            if map
                .get_tile_from_coord(&t.translation)
                .map_or(false, |tile| visible_tiles.contains(&(tile.x, tile.y)))
            {
                // The ant is visible, reposition it
                *ant_t = *t;
            } else {
                // The ant is no longer visible, despawn it
                despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
            }
        } else {
            // The ant is no longer in the population (died), despawn it
            despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
        }
    }

    // Spawn new ants if they are on a visible tile
    for (ant_t, ant) in population.0.values() {
        // If the ant is new in the population, spawn it if it's visible
        if !current_population.contains(&ant.id)
            && map
                .get_tile_from_coord(&ant_t.translation)
                .map_or(false, |tile| visible_tiles.contains(&(tile.x, tile.y)))
        {
            spawn_ant_ev.send(SpawnAntEv {
                ant: ant.clone(),
                transform: ant_t.clone(),
            });
        }
    }
}
