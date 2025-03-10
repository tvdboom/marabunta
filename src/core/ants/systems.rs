use crate::core::ants::components::*;
use crate::core::ants::events::{DamageAntEv, DespawnAntEv, SpawnAntEv, SpawnEggEv};
use crate::core::ants::utils::walk;
use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::map::events::SpawnTileEv;
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::map::utils::reveal_tiles;
use crate::core::player::Player;
use crate::core::resources::{GameSettings, Population};
use crate::core::utils::{collision, scale_duration};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::utils::HashSet;
use rand::prelude::IndexedRandom;
use rand::{rng, Rng};
use std::f32::consts::PI;

pub fn hatch_eggs(
    mut egg_q: Query<(Entity, &mut Egg, &Transform)>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
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
            let mut ant = AntCmp::new(&egg.ant, player.id);

            // If the egg was damaged the ant spawns with the same health ratio
            if egg.health < egg.max_health {
                ant.health = (egg.health / egg.max_health) * ant.max_health;
            }

            spawn_ant_ev.send(SpawnAntEv {
                ant,
                transform: egg_t.clone(),
            });

            despawn_ant_ev.send(DespawnAntEv { entity: egg_e });
        }
    }
}

pub fn animate_ants(
    mut ant_q: Query<(&mut Sprite, &AntCmp, &mut AnimationCmp)>,
    mut damage_ev: EventWriter<DamageAntEv>,
    game_settings: Res<GameSettings>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    for (mut sprite, ant, mut animation) in ant_q.iter_mut() {
        if ant.animation() == animation.animation {
            // If the ant's action matches the animation, continue the frames
            animation
                .timer
                .tick(scale_duration(time.delta(), game_settings.speed));

            if animation.timer.just_finished() {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = if atlas.index == animation.last_index {
                        if matches!(ant.action, Action::Die(_)) {
                            atlas.index // Remain at last frame when dead
                        } else {
                            0
                        }
                    } else {
                        atlas.index + 1
                    };

                    // Apply damage halfway the animation
                    if let Action::Attack(id) = ant.action {
                        if atlas.index == animation.last_index / 2 + 1 {
                            damage_ev.send(DamageAntEv {
                                attacker: ant.id,
                                defender: id,
                            });
                        }
                    }
                }
            }
        } else {
            // Else adjust the atlas
            let atlas = assets.atlas(&format!(
                "{}_{}",
                ant.kind.to_snake(),
                ant.animation().to_snake()
            ));

            *sprite = Sprite {
                image: atlas.image,
                texture_atlas: Some(atlas.texture),
                ..default()
            };

            let interval = if ant.animation() == Animation::Walk {
                ant.kind.interval(&ant.animation()) * DEFAULT_WALK_SPEED / ant.speed
            } else {
                ant.kind.interval(&ant.animation())
            };

            *animation = AnimationCmp {
                animation: ant.animation(),
                timer: Timer::from_seconds(interval, TimerMode::Repeating),
                last_index: atlas.last_index,
            };
        }
    }
}

pub fn resolve_digging(
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    mut tile_q: Query<&mut Tile>,
    mut map: ResMut<Map>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    game_settings: Res<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
) {
    for mut tile in tile_q.iter_mut() {
        // Select ants that were digging on that tile
        let mut ants: Vec<_> = ant_q
            .iter_mut()
            .filter(|(_, a)| {
                player.owns(a) && matches!(&a.action, Action::Dig(t) if t.equals(&tile))
            })
            .collect();

        if !ants.is_empty() {
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
                if !tile.visible.contains(&player.id) && rng().random::<f32>() < TILE_ENEMY_CHANCE {
                    // Spawn an enemy on the newly dug tile
                    spawn_ant_ev.send(SpawnAntEv {
                        ant: AntCmp::new(&Ant::BlackScorpion, 0),
                        transform: Transform {
                            translation: Map::get_coord_from_xy(tile.x, tile.y).extend(0.),
                            rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                            ..default()
                        },
                    });
                }

                map.find_and_replace_tile(&tile, &directions, player.id);

                // Set digging ants onto a new task
                ants.iter_mut().for_each(|(_, ant)| {
                    ant.action = if rng().random::<f32>() < SAME_TUNNEL_DIG_CHANCE {
                        if let Some(loc) = map.random_dig_loc(Some(&tile), player.id) {
                            Action::Walk(loc)
                        } else {
                            // If there are no digging locations on the tile, select a random one
                            Action::Idle
                        }
                    } else {
                        Action::Idle
                    }
                });
            }
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
        .filter(|(_, a)| player.owns(a) && a.action == Action::Harvest)
    {
        if let Some(tile) = map.get_tile_mut_from_coord(&ant_t.translation) {
            if let Some(ref mut leaf) = &mut tile.leaf {
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

                if leaf.quantity == 0. {
                    tile.leaf = None;
                }
            } else {
                ant.action = Action::Idle;
            }
        }
    }
}

pub fn resolve_healing(
    mut ant_q: Query<(&Transform, &mut AntCmp)>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
) {
    for (ant_t, mut ant) in ant_q
        .iter_mut()
        .filter(|(_, a)| player.owns(a) && a.action == Action::Heal)
    {
        if let Some(tile) = map.get_tile_mut_from_coord(&ant_t.translation) {
            if let Some(ref mut leaf) = &mut tile.leaf {
                let heal =
                    (HEAL_SPEED_RATIO * ant.max_health * game_settings.speed * time.delta_secs())
                        .min(leaf.quantity);

                let health = (ant.health + heal).min(ant.max_health);
                let healed = health - ant.health;
                ant.health = health;
                leaf.quantity -= healed;

                if leaf.quantity == 0. {
                    tile.leaf = None;
                }
            }

            if ant.health == ant.max_health || tile.leaf.is_none() {
                ant.behavior = AntCmp::new(&ant.kind, player.id).behavior;
                ant.action = Action::Idle;
            }
        }
    }
}

pub fn resolve_attack_action(
    mut ant_q: Query<(&Transform, &mut AntCmp)>,
    egg_q: Query<(&Transform, &Egg)>,
    player: Res<Player>,
) {
    let enemies: Vec<_> = ant_q
        .iter()
        .filter_map(|(t, a)| (a.health > 0.).then(|| (a.id, t.translation, a.scaled_size())))
        .chain(
            egg_q
                .iter()
                .map(|(t, e)| (e.id, t.translation, e.scaled_size())),
        )
        .collect();

    for (ant_t, mut ant) in ant_q.iter_mut().filter(|(_, a)| player.owns(a)) {
        if let Action::Attack(id) = ant.action {
            if let Some((_, pos_t, size_t)) = enemies.iter().find(|(i, _, _)| *i == id) {
                if !collision(&ant_t.translation, &ant.scaled_size(), pos_t, size_t) {
                    // The enemy is not adjacent anymore
                    ant.action = Action::TargetedWalk(id);
                }
            } else {
                // The enemy is dead
                ant.action = Action::Idle;
            }
        }
    }
}

pub fn resolve_brood_action(
    mut ant_q: Query<(&Transform, &mut AntCmp)>,
    mut spawn_egg_ev: EventWriter<SpawnEggEv>,
    game_settings: Res<GameSettings>,
    mut player: ResMut<Player>,
    time: Res<Time>,
) {
    for (ant_t, mut ant) in ant_q.iter_mut() {
        if player.owns(&ant) {
            if let Action::Brood(timer) = &mut ant.action {
                timer.tick(scale_duration(time.delta(), game_settings.speed));

                if timer.just_finished() {
                    if let Some(ant_queue) = player.queue.pop_front() {
                        spawn_egg_ev.send(SpawnEggEv {
                            ant: ant_queue,
                            transform: *ant_t,
                        });
                    }

                    ant.action = Action::Idle;
                }
            }
        }
    }
}

pub fn resolve_die_action(
    mut ant_q: Query<(Entity, &mut AntCmp)>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
    game_settings: Res<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
) {
    for (ant_e, mut ant) in ant_q.iter_mut().filter(|(_, a)| player.owns(a)) {
        if let Action::Die(timer) = &mut ant.action {
            timer.tick(scale_duration(time.delta(), game_settings.speed));

            if timer.just_finished() {
                despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
            }
        }
    }
}

pub fn resolve_idle_action(
    mut ant_q: Query<(&Transform, &mut AntCmp)>,
    egg_q: Query<(&Transform, &Egg)>,
    player: Res<Player>,
    map: Res<Map>,
) {
    let queen_id = ant_q
        .iter()
        .find(|(_, a)| player.owns(a) && a.kind == Ant::BlackQueen)
        .map(|(_, a)| a.id)
        .unwrap_or_default();

    let enemies: Vec<_> = ant_q
        .iter()
        .filter_map(|(t, a)| {
            if a.health > 0. {
                Some((a.clone(), t.translation))
            } else {
                None
            }
        })
        .collect();

    for (ant_t, mut ant) in ant_q
        .iter_mut()
        .filter(|(_, a)| player.owns(a) && a.action == Action::Idle)
    {
        // If hurt, go heal to a leaf
        if ant.health < ant.max_health && ant.kind != Ant::BlackQueen && !ant.kind.is_monster() {
            if let Some(loc) = map.closest_leaf_loc(&ant_t.translation, player.id) {
                ant.behavior = Behavior::Heal;
                ant.action = Action::Walk(loc);
                return;
            }
        }

        ant.action = match ant.behavior {
            Behavior::Attack => {
                // Select actual enemies from this ant
                let enemies: Vec<_> = enemies
                    .iter()
                    .filter(|(a, _)| ant.team != a.team)
                    .map(|(a, t)| (a.id, t))
                    .chain(
                        egg_q
                            .iter()
                            .filter(|(_, e)| ant.team != e.team)
                            .map(|(t, e)| (e.id, &t.translation)),
                    )
                    .collect();

                if enemies.is_empty() {
                    map.random_enemy_loc(player.id)
                        .or_else(|| map.random_loc(player.id, false))
                        .map(Action::Walk)
                        .unwrap()
                } else if rng().random::<f32>() < ATTACK_CLOSEST_ENEMY_CHANCE {
                    Action::TargetedWalk(
                        enemies
                            .iter()
                            .min_by_key(|(_, t)| t.distance(ant_t.translation) as u32)
                            .unwrap()
                            .0,
                    )
                } else {
                    Action::TargetedWalk(enemies.choose(&mut rng()).unwrap().0)
                }
            }
            Behavior::Brood => Action::Walk(map.random_loc(player.id, true).unwrap()),
            Behavior::Dig => Action::Walk(
                map.random_dig_loc(None, player.id)
                    .unwrap_or(map.random_loc(player.id, false).unwrap()),
            ),
            Behavior::Harvest => {
                if ant.carry < ant.max_carry / 2. {
                    Action::Walk(
                        map.random_leaf_loc(player.id)
                            .unwrap_or(map.random_loc(player.id, false).unwrap()),
                    )
                } else {
                    Action::TargetedWalk(queen_id)
                }
            }
            Behavior::Wander => Action::Walk(map.random_loc(player.id, false).unwrap()),
            _ => unreachable!(),
        }
    }
}

pub fn resolve_targeted_walk_action(
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    egg_q: Query<(&Transform, &Egg), Without<AntCmp>>,
    mut player: ResMut<Player>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    let ant_elem: Vec<_> = ant_q
        .iter()
        .map(|(t, a)| (a.id, a.team, t.translation, a.scaled_size()))
        .chain(
            egg_q
                .iter()
                .map(|(t, e)| (e.id, e.team, t.translation, e.scaled_size())),
        )
        .collect();

    for (mut ant_t, mut ant) in ant_q.iter_mut() {
        if player.owns(&ant) {
            if let Action::TargetedWalk(id) = ant.action {
                if let Some((_, team_t, pos_t, size_t)) =
                    ant_elem.iter().find(|(i, _, _, _)| i == &id)
                {
                    if !collision(&ant_t.translation, &ant.scaled_size(), pos_t, size_t) {
                        let speed = ant.speed
                            * game_settings.speed
                            * time.delta_secs()
                            * if ant.can_fly { FLY_SPEED_FACTOR } else { 1. };

                        walk(
                            &mut ant_t,
                            &map.get_loc(pos_t),
                            speed,
                            &mut map,
                            &game_settings,
                            &time,
                        );
                    } else if *team_t == ant.team {
                        // Ant reached the queen -> deposit food
                        player.food += ant.carry;
                        ant.carry = 0.;
                        ant.action = Action::Idle;
                    } else {
                        // Ant reached the enemy
                        let d = -ant_t.translation + *pos_t;

                        // Rotate towards the target and attack
                        let rotation = ant_t.rotation.rotate_towards(
                            Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                            3. * game_settings.speed * time.delta_secs(),
                        );

                        ant.action = if ant_t.rotation == rotation {
                            Action::Attack(id)
                        } else {
                            ant_t.rotation = rotation;
                            Action::TargetedWalk(id)
                        };
                    }
                } else {
                    // The target doesn't exist anymore
                    ant.action = Action::Idle;
                }
            }
        }
    }
}

pub fn resolve_walk_action(
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    player: Res<Player>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (mut ant_t, mut ant) in ant_q.iter_mut().filter(|(_, a)| player.owns(a)) {
        if let Action::Walk(target_loc) = ant.action {
            let current_loc = map.get_loc(&ant_t.translation);
            if current_loc != target_loc {
                walk(
                    &mut ant_t,
                    &target_loc,
                    ant.speed * game_settings.speed * time.delta_secs(),
                    &mut map,
                    &game_settings,
                    &time,
                );
            } else {
                // Ant reached the target -> continue with default action
                ant.action = match ant.behavior {
                    Behavior::Brood => {
                        if !player.queue.is_empty() {
                            Action::Brood(Timer::from_seconds(BROODING_TIME, TimerMode::Once))
                        } else {
                            Action::Idle
                        }
                    }
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
                    Behavior::Harvest | Behavior::Heal => {
                        // The leaf could have been harvested completely while getting there
                        if map
                            .get_tile(current_loc.x, current_loc.y)
                            .unwrap()
                            .leaf
                            .is_some()
                        {
                            // Ant reached the leaf => turn towards it
                            let current_loc = map.get_loc(&ant_t.translation);
                            let d = -ant_t.translation
                                + Map::get_coord_from_xy(current_loc.x, current_loc.y)
                                    .extend(ant_t.translation.z);

                            let rotation = ant_t.rotation.rotate_towards(
                                Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                                3. * game_settings.speed * time.delta_secs(),
                            );

                            if ant_t.rotation != rotation {
                                ant_t.rotation = rotation;
                                Action::Walk(target_loc)
                            } else if ant.behavior == Behavior::Harvest {
                                Action::Harvest
                            } else {
                                Action::Heal
                            }
                        } else {
                            Action::Idle
                        }
                    }
                    _ => Action::Idle,
                };
            }
        }
    }
}

pub fn update_ant_components(
    ant_q: Query<
        (Entity, &Transform, &AntCmp),
        (Without<AntHealthWrapperCmp>, Without<AntHealthCmp>),
    >,
    egg_q: Query<(Entity, &Transform, &Egg), (Without<AntHealthWrapperCmp>, Without<AntHealthCmp>)>,
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
                *leaf_v = if ant.carry >= ant.max_carry / 2. {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }

            if let Ok((wrapper_e, mut wrapper_t, mut wrapper_v)) = wrapper_q.get_mut(child) {
                // Show the health bar when the ant is damaged
                if ant.health > 0. && ant.health < ant.max_health {
                    *wrapper_v = Visibility::Inherited;

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

    for (egg_e, egg_t, egg) in egg_q.iter() {
        for child in children_q.iter_descendants(egg_e) {
            if let Ok((wrapper_e, mut wrapper_t, mut wrapper_v)) = wrapper_q.get_mut(child) {
                // Show the health bar when the egg is damaged
                if egg.health > 0. && egg.health < egg.max_health {
                    *wrapper_v = Visibility::Inherited;

                    // Place the health bar on top of the egg on a distance dependent on the egg's rotation
                    wrapper_t.translation = Vec3::new(
                        egg.size().x * 0.5 * egg_t.rotation.to_euler(EulerRot::ZXY).0.sin(),
                        egg.size().y * 0.5 * egg_t.rotation.to_euler(EulerRot::ZXY).0.cos(),
                        0.1,
                    );

                    for child in children_q.iter_descendants(wrapper_e) {
                        if let Ok((mut health_t, mut health_s)) = health_q.get_mut(child) {
                            if let Some(size) = health_s.custom_size.as_mut() {
                                let full_size = egg.size().x * 0.77;
                                size.x = full_size * egg.health / egg.max_health;
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
    mut ant_q: Query<(Entity, &mut Transform, &mut Visibility, &AntCmp)>,
    mut spawn_tile_ev: EventWriter<SpawnTileEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
    player: Res<Player>,
    mut map: ResMut<Map>,
    population: Res<Population>,
) {
    let mut visible_tiles = HashSet::new();

    // Calculate all tiles currently visible by the player
    ant_q
        .iter()
        .filter(|(_, _, _, a)| player.controls(a) && a.health > 0.)
        .for_each(|(_, ant_t, _, _)| {
            let current_tile = map.get_tile_from_coord(&ant_t.translation).unwrap();
            visible_tiles.extend(reveal_tiles(current_tile, &map, None, 0))
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
    for (ant_e, mut ant_t, mut ant_v, ant) in ant_q
        .iter_mut()
        .filter(|(_, _, _, a)| a.owner != player.id || a.kind.is_monster())
    {
        current_population.push(ant.id);
        if let Some((t, _)) = population.0.values().find(|(_, a)| a.id == ant.id) {
            // The ant is already on the map
            if map
                .get_tile_from_coord(&t.translation)
                .map_or(false, |tile| visible_tiles.contains(&(tile.x, tile.y)))
            {
                // The ant is visible, reposition and show it
                *ant_t = *t;
                *ant_v = Visibility::Visible;
            } else {
                // The ant is no longer visible, hide it
                *ant_v = Visibility::Hidden;
            }
        } else {
            if ant.kind.is_monster() {
                if map
                    .get_tile_from_coord(&ant_t.translation)
                    .map_or(false, |tile| visible_tiles.contains(&(tile.x, tile.y)))
                {
                    // The monster is visible, show it
                    *ant_v = Visibility::Visible;
                } else {
                    // The monster is no longer visible, hide it
                    *ant_v = Visibility::Hidden;
                }
            } else {
                // The ant is no longer in the population (died), despawn it
                despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
            }
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
