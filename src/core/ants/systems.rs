use crate::core::ants::components::*;
use crate::core::ants::events::*;
use crate::core::ants::selection::AntSelection;
use crate::core::ants::utils::walk;
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioEv;
use crate::core::constants::*;
use crate::core::game_settings::GameSettings;
use crate::core::map::events::SpawnTileEv;
use crate::core::map::map::Map;
use crate::core::map::tile::{Leaf, Tile};
use crate::core::map::utils::reveal_tiles;
use crate::core::player::Players;
use crate::core::resources::Resources;
use crate::core::traits::Trait;
use crate::core::utils::{collision, scale_duration};
use bevy::prelude::*;
use bevy::utils::hashbrown::{HashMap, HashSet};
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::{rng, Rng};
use std::f32::consts::PI;
use strum::IntoEnumIterator;

pub fn hatch_eggs(
    mut egg_q: Query<(Entity, &mut Egg, &Transform)>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    time: Res<Time>,
) {
    for (egg_e, mut egg, egg_t) in egg_q.iter_mut() {
        let time = scale_duration(
            scale_duration(time.delta(), game_settings.speed),
            if players.get(egg.owner).has_trait(&Trait::Breeding) {
                HATCH_SPEED_FACTOR
            } else {
                1.
            },
        );

        egg.timer.tick(time);

        if egg.timer.just_finished() {
            spawn_ant_ev.send(SpawnAntEv {
                ant: AntCmp {
                    health: (egg.health / egg.max_health) * egg.ant.max_health, // Keep the health ratio
                    ..egg.ant.clone()
                },
                transform: egg_t.clone(),
            });

            despawn_ant_ev.send(DespawnAntEv { entity: egg_e });
        }
    }
}

pub fn animate_ants(
    mut ant_q: Query<(Entity, &mut Sprite, &AntCmp, &mut AnimationCmp)>,
    mut damage_ev: EventWriter<DamageAntEv>,
    game_settings: Res<GameSettings>,
    assets: Local<WorldAssets>,
    time: Res<Time>,
) {
    for (ant_e, mut sprite, ant, mut animation) in ant_q.iter_mut() {
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
                    if let Action::Attack(entity) = ant.action {
                        if atlas.index == animation.last_index / 2 + 1 {
                            damage_ev.send(DamageAntEv {
                                attacker: ant_e,
                                defender: entity,
                            });
                        }
                    }
                }
            }
        } else {
            // Else adjust the atlas
            let atlas = assets.atlas(&ant.atlas(&ant.animation()));

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
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    time: Res<Time>,
) {
    for mut tile in tile_q.iter_mut() {
        // Select ants that were digging on that tile
        let mut ants: Vec<_> = ant_q
            .iter_mut()
            .filter(|(_, a)| matches!(&a.action, Action::Dig(t) if t.equals(&tile)))
            .collect();

        if !ants.is_empty() {
            let mut terraform = 0.;

            // Turn ants towards the direction they are digging
            let mut directions = HashSet::new();
            ants.iter_mut().for_each(|(t, a)| {
                let d = map.get_loc(&t.translation).get_direction();
                t.rotation = t.rotation.rotate_towards(
                    Quat::from_rotation_z(d.degrees()),
                    2. * game_settings.speed * time.delta_secs(),
                );
                directions.insert(d);

                terraform += DIG_SPEED
                    * game_settings.speed
                    * time.delta_secs()
                    * if players.get(a.owner).has_trait(&Trait::Tunneling) {
                        TUNNEL_SPEED_FACTOR
                    } else {
                        1.
                    };
            });

            if tile.terraform > terraform {
                tile.terraform -= terraform;
            } else {
                // Possibly spawn a scorpion on the newly dug tile
                if let Some(enemy) = match rng().random::<f32>() {
                    0.95..0.99 => Some(Ant::BlackScorpion),
                    0.99..=1. => Some(Ant::YellowScorpion),
                    _ => None,
                } {
                    play_audio_ev.send(PlayAudioEv {
                        name: "warning",
                        volume: 0.5,
                    });

                    spawn_ant_ev.send(SpawnAntEv {
                        ant: AntCmp::base(&enemy),
                        transform: Transform {
                            translation: Map::get_coord_from_xy(tile.x, tile.y).extend(0.),
                            rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                            ..default()
                        },
                    });
                }

                map.find_and_replace_tile(
                    &tile,
                    &directions,
                    ants.iter().map(|(_, a)| a.owner).collect::<Vec<_>>(),
                );

                // Set digging ants onto a new task
                ants.iter_mut().for_each(|(_, ant)| {
                    ant.action = if matches!(ant.command, Some(Behavior::Dig(_)))
                        || rng().random::<f32>() >= SAME_TUNNEL_DIG_CHANCE
                    {
                        Action::Idle
                    } else if let Some(loc) = map.random_dig_loc(Some(&tile), ant.owner) {
                        Action::Walk(loc)
                    } else {
                        // If there are no digging locations on the tile, select a random one
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
    players: Res<Players>,
    time: Res<Time>,
) {
    for (ant_t, mut ant) in ant_q.iter_mut().filter(|(_, a)| {
        a.action == Action::Harvest
            && matches!(
                a.get_behavior(),
                Behavior::Harvest(_) | Behavior::HarvestRandom
            )
    }) {
        if let Some(tile) = map.get_tile_mut_from_coord(&ant_t.translation) {
            if let Some(ref mut leaf) = &mut tile.leaf {
                let player = players.get(ant.owner);

                let leaves = (HARVEST_SPEED
                    * game_settings.speed
                    * time.delta_secs()
                    * if player.has_trait(&Trait::Harvest) {
                        HARVEST_SPEED_FACTOR
                    } else {
                        1.
                    }
                    * if player.has_trait(&Trait::Warlike) {
                        HARVEST_DECREASE_FACTOR
                    } else {
                        1.
                    })
                .min(leaf.quantity);

                if ant.carry.leaves + leaves > ant.max_carry.leaves {
                    ant.carry.leaves = ant.max_carry.leaves;
                    leaf.quantity -= ant.max_carry.leaves - ant.carry.leaves;
                    ant.action = Action::Idle;
                } else {
                    ant.carry.leaves += leaves;
                    leaf.quantity -= leaves;
                }

                if leaf.quantity == 0. {
                    tile.leaf = None;
                }
            } else {
                ant.command = None;
                ant.action = Action::Idle;
            }
        }
    }
}

pub fn resolve_harvesting_corpse(
    mut ant_q: Query<(Entity, &mut AntCmp)>,
    corpse_q: Query<Entity, With<Corpse>>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    time: Res<Time>,
) {
    for (_, mut ant) in ant_q
        .iter_mut()
        .filter(|(_, a)| a.action == Action::Harvest)
    {
        if let Some(Behavior::HarvestCorpse(entity)) = ant.command {
            if corpse_q.get(entity).is_ok() {
                let player = players.get(ant.owner);

                let nutrients = HARVEST_SPEED
                    * game_settings.speed
                    * time.delta_secs()
                    * if player.has_trait(&Trait::Harvest) {
                        HARVEST_SPEED_FACTOR
                    } else {
                        1.
                    }
                    * if player.has_trait(&Trait::Warlike) {
                        HARVEST_DECREASE_FACTOR
                    } else {
                        1.
                    };

                if ant.carry.nutrients + nutrients > ant.max_carry.nutrients {
                    ant.carry.nutrients = ant.max_carry.nutrients;
                    ant.action = Action::Idle;
                } else {
                    ant.carry.nutrients += nutrients;
                }
            } else {
                ant.command = None;
                ant.action = Action::Idle;
            }
        }
    }
}

pub fn resolve_healing(
    mut ant_q: Query<(Entity, &mut AntCmp)>,
    corpse_q: Query<Entity, With<Corpse>>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (_, mut ant) in ant_q.iter_mut().filter(|(_, a)| a.action == Action::Heal) {
        let heal = HEAL_SPEED_RATIO * ant.max_health * game_settings.speed * time.delta_secs();

        if ant.kind == Ant::Queen {
            // A queen heals herself very slowly (but no corps required)
            ant.health = (ant.health + heal * 0.1).min(ant.max_health);
        } else if let Behavior::Heal(entity) = ant.behavior {
            if corpse_q.get(entity).is_ok() {
                ant.health = (ant.health + heal).min(ant.max_health);
            } else {
                // The corpse doesn't exist anymore
                ant.behavior = AntCmp::base(&ant.kind).behavior;
                ant.action = Action::Idle;
            }
        }

        if ant.health == ant.max_health {
            ant.behavior = AntCmp::base(&ant.kind).behavior;
            ant.action = Action::Idle;
        }
    }
}

pub fn resolve_pre_action(
    mut ant_q: Query<(Entity, &Transform, &Sprite, &mut AntCmp)>,
    corpse_q: Query<(Entity, &Transform), With<Corpse>>,
    mut map: ResMut<Map>,
    players: Res<Players>,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
) {
    let enemies = ant_q
        .iter()
        .filter_map(|(e, t, s, a)| {
            (a.health > 0.).then_some((e, a.team, a.action.clone(), t.clone(), s.clone()))
        })
        .collect::<Vec<_>>();

    'ant: for (_, ant_t, ant_s, mut ant) in ant_q
        .iter_mut()
        .filter(|(_, _, _, a)| !matches!(a.action, Action::Attack(_) | Action::Die(_)))
    {
        for (enemy_e, enemy_team, enemy_a, enemy_t, enemy_s) in enemies.iter() {
            if ant.team != *enemy_team {
                // The queen attacks enemies in the base (except when wandering)
                // Protecting ants attack enemies attacking the protected ant
                // All ants attack when adjacent
                if (ant.kind == Ant::Queen
                    && !players.get(ant.owner).has_trait(&Trait::WanderingQueen)
                    && ant.command.is_none()
                    && map
                        .get_tile_from_coord(&enemy_t.translation)
                        .unwrap()
                        .base
                        .filter(|b| *b == ant.owner)
                        .is_some())
                    || matches!(
                        (enemy_a, &ant.command),
                        (Action::Attack(e1), Some(Behavior::ProtectAnt(e2))) if e1 == e2
                    )
                    || collision((&ant_t, &ant_s), (enemy_t, enemy_s), &images, &atlases)
                {
                    ant.action = Action::TargetedWalk(*enemy_e);
                    continue 'ant;
                }
            }
        }

        // Worker ants collect nutrients when close to a corpse
        if ant.kind == Ant::Worker {
            for (corpse_e, corpse_t) in corpse_q.iter() {
                let ant_loc = map.get_loc(&ant_t.translation);
                let corpse_loc = map.get_loc(&corpse_t.translation);
                if map.distance(&ant_loc, &corpse_loc) <= MAX_DISTANCE_PROTECT {
                    ant.command = Some(Behavior::HarvestCorpse(corpse_e));
                    ant.action = Action::TargetedWalk(corpse_e);
                    continue 'ant;
                }
            }
        }
    }
}

pub fn resolve_attack_action(
    mut ant_q: Query<(Entity, &Transform, &Visibility, &Sprite, &mut AntCmp)>,
    egg_q: Query<(Entity, &Transform, &Sprite), With<Egg>>,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
) {
    let enemies: HashMap<_, _> = ant_q
        .iter()
        .filter_map(|(e, t, v, s, a)| {
            (a.health > 0. && v != Visibility::Hidden).then_some((e, (t.clone(), s.clone())))
        })
        .chain(egg_q.iter().map(|(e, t, s)| (e, (t.clone(), s.clone()))))
        .collect();

    for (_, ant_t, _, ant_s, mut ant) in ant_q.iter_mut() {
        if let Action::Attack(entity) = ant.action {
            if let Some((enemy_t, enemy_s)) = enemies.get(&entity) {
                if !collision((ant_t, ant_s), (enemy_t, enemy_s), &images, &atlases) {
                    // The enemy is not adjacent anymore
                    ant.action = Action::TargetedWalk(entity);
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
    mut players: ResMut<Players>,
    time: Res<Time>,
) {
    for (ant_t, mut ant) in ant_q.iter_mut() {
        if let Action::Brood(timer) = &mut ant.action {
            timer.tick(scale_duration(time.delta(), game_settings.speed));

            if timer.just_finished() {
                let player = players.get_mut(ant.owner);

                if let Some(ant_queue) = player.queue.pop_front() {
                    spawn_egg_ev.send(SpawnEggEv {
                        ant: AntCmp::new(&ant_queue, &player),
                        transform: *ant_t,
                    });
                }

                ant.action = Action::Idle;
            }
        }
    }
}

pub fn resolve_die_action(
    mut ant_q: Query<(Entity, &mut AntCmp)>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (ant_e, mut ant) in ant_q.iter_mut() {
        if let Action::Die(timer) = &mut ant.action {
            timer.tick(scale_duration(time.delta(), game_settings.speed));

            if timer.just_finished() {
                despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
            }
        }
    }
}

pub fn resolve_idle_action(
    mut ant_q: Query<(Entity, &Transform, &Visibility, &mut AntCmp)>,
    corpse_q: Query<(Entity, &Transform), With<Corpse>>,
    egg_q: Query<(Entity, &Transform, &Egg)>,
    leaf_q: Query<(Entity, &Transform), With<Leaf>>,
    players: Res<Players>,
    mut map: ResMut<Map>,
) {
    let queen_e = ant_q
        .iter()
        .find(|(_, _, _, a)| a.kind == Ant::Queen)
        .map(|(e, _, _, _)| e)
        .unwrap_or(Entity::PLACEHOLDER);

    let ants: HashMap<_, _> = ant_q
        .iter()
        .filter_map(|(e, t, v, a)| {
            (a.health > 0. && v != Visibility::Hidden).then_some((e, (t.translation, a.clone())))
        })
        .collect();

    for (_, ant_t, _, mut ant) in ant_q
        .iter_mut()
        .filter(|(_, _, _, a)| a.action == Action::Idle)
    {
        let player = players.get(ant.owner);

        let current_loc = map.get_loc(&ant_t.translation);

        // If hurt, go heal to the nearest corpse
        if ant.health < ant.max_health && ant.kind.is_ant() {
            if ant.kind != Ant::Queen {
                if let Some((entity, _)) = corpse_q.iter().min_by_key(|(_, t)| {
                    let loc = map.get_loc(&t.translation);
                    map.distance(&current_loc, &loc)
                }) {
                    ant.behavior = Behavior::Heal(entity);
                    ant.action = Action::TargetedWalk(entity);
                    return;
                }
            } else if player.has_trait(&Trait::HealingQueen) {
                ant.action = Action::Heal;
                return;
            }
        }

        ant.action = match ant.get_behavior() {
            Behavior::Attack => {
                // Select enemies from this ant and calculate distance weight
                let enemies: Vec<_> = ants
                    .iter()
                    .filter(|(_, (_, a))| ant.team != a.team)
                    .map(|(e, (t, _))| (*e, t))
                    .chain(
                        egg_q
                            .iter()
                            .filter(|(_, _, e)| ant.team != e.team)
                            .map(|(e, t, _)| (e, &t.translation)),
                    )
                    .collect();

                if enemies.is_empty() {
                    map.random_enemy_loc(ant.owner)
                        .or_else(|| map.random_loc(ant.owner, false))
                        .map(Action::Walk)
                        .unwrap()
                } else {
                    // Attack chance decreases exponentially with distance
                    let index = WeightedIndex::new(enemies.iter().map(|(_, t)| {
                        let loc = map.get_loc(&t);
                        1. / map.distance(&current_loc, &loc).pow(2) as f32
                    }))
                    .unwrap();

                    Action::TargetedWalk(enemies[index.sample(&mut rng())].0)
                }
            }
            Behavior::Brood => Action::Walk(if player.has_trait(&Trait::WanderingQueen) {
                map.random_loc_max_distance(ant.owner, &current_loc, 10)
                    .unwrap()
            } else {
                map.random_loc(ant.owner, true).unwrap()
            }),
            Behavior::Dig(loc) => map
                .find_tunnel(&current_loc, &loc)
                .and_then(|path| path.into_iter().find(|l| !map.is_walkable(l)))
                .map(|loc| Action::Walk(loc))
                .unwrap_or_else(|| {
                    ant.command = None;
                    Action::Idle
                }),
            Behavior::DigRandom => Action::Walk(
                map.random_dig_loc(None, ant.owner)
                    .unwrap_or(map.random_loc(ant.owner, false).unwrap()),
            ),
            Behavior::Harvest(entity) => {
                Action::TargetedWalk(if ant.carry.leaves < ant.max_carry.leaves / 2. {
                    *entity
                } else {
                    queen_e
                })
            }
            Behavior::HarvestCorpse(entity) => {
                Action::TargetedWalk(if ant.carry.nutrients < ant.max_carry.nutrients / 2. {
                    *entity
                } else {
                    queen_e
                })
            }
            Behavior::HarvestRandom => {
                if leaf_q.iter().count() > 0 {
                    // If above half carry capacity or no more leaves, walk to the queen
                    Action::TargetedWalk(if ant.carry < ant.max_carry / 2. {
                        let index = WeightedIndex::new(leaf_q.iter().map(|(_, t)| {
                            let loc = map.get_loc(&t.translation);
                            1. / map.distance(&current_loc, &loc) as f32
                        }))
                        .unwrap();

                        leaf_q.iter().nth(index.sample(&mut rng())).unwrap().0
                    } else {
                        queen_e
                    })
                } else {
                    // If there are no leaves left, wander around
                    Action::Walk(map.random_loc(ant.owner, false).unwrap())
                }
            }
            Behavior::Heal(_) => {
                // Reset behavior
                ant.behavior = AntCmp::base(&ant.kind).behavior;
                Action::Idle
            }
            Behavior::ProtectAnt(entity) => {
                // Walk randomly but stay close to the protected ant
                if let Some((t, _)) = ants.get(entity) {
                    let loc = map.get_loc(t);
                    Action::Walk(
                        map.random_loc_max_distance(ant.owner, &loc, MAX_DISTANCE_PROTECT)
                            .unwrap(),
                    )
                } else {
                    // Entity to protect doesn't exist anymore
                    ant.command = None;
                    Action::Idle
                }
            }
            Behavior::ProtectLoc(loc) => {
                // Walk randomly but stay close to the location
                Action::Walk(
                    map.random_loc_max_distance(ant.owner, loc, MAX_DISTANCE_PROTECT)
                        .unwrap(),
                )
            }
            Behavior::Wander => Action::Walk(map.random_loc(ant.owner, false).unwrap()),
        };
    }
}

pub fn resolve_targeted_walk_action(
    mut ant_q: Query<(Entity, &mut Transform, &Sprite, &mut AntCmp)>,
    sprite_q: Query<(Entity, &GlobalTransform, &Sprite, &TeamCmp)>,
    corpse_q: Query<Entity, With<Corpse>>,
    mut players: ResMut<Players>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
) {
    for (_, mut ant_t, ant_s, mut ant) in ant_q.iter_mut() {
        if let Action::TargetedWalk(entity) = ant.action {
            if let Ok((_, target_t, target_s, team)) = sprite_q.get(entity) {
                let player = players.get_mut(ant.owner);

                let target_t = target_t.compute_transform();

                let current_loc = map.get_loc(&ant_t.translation);
                let target_loc = map.get_loc(&target_t.translation);

                if !collision((&ant_t, ant_s), (&target_t, target_s), &images, &atlases)
                    && current_loc != target_loc
                {
                    // The ant isn't adjacent to the target yet -> keep walking
                    let speed = ant.speed
                        * game_settings.speed
                        * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED)
                        * if ant.kind.can_fly() {
                            FLY_SPEED_FACTOR
                        } else {
                            1.
                        }
                        * if player.has_trait(&Trait::Haste) {
                            HASTE_SPEED_FACTOR
                        } else {
                            1.
                        };

                    walk(
                        &mut ant_t,
                        &target_loc,
                        speed,
                        &mut map,
                        &game_settings,
                        &time,
                    );
                } else if team.0 == ant.team && corpse_q.get(entity).is_err() {
                    if matches!(
                        ant.get_behavior(),
                        Behavior::Harvest(_) | Behavior::HarvestCorpse(_) | Behavior::HarvestRandom
                    ) {
                        // Ant reached the queen -> deposit food
                        player.resources += &ant.carry;
                        ant.carry = Resources::default();
                    }

                    // Ant reached the target -> continue with default action
                    ant.action = Action::Idle;
                } else {
                    // Ant reached an enemy, a leaf or a corpse
                    let d = -ant_t.translation + target_t.translation;

                    // Rotate towards the target and attack
                    let rotation = ant_t.rotation.rotate_towards(
                        Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                        3. * game_settings.speed * time.delta_secs(),
                    );

                    ant.action = if ant_t.rotation.angle_between(rotation) < 0.01 {
                        match ant.get_behavior() {
                            Behavior::Harvest(_)
                            | Behavior::HarvestCorpse(_)
                            | Behavior::HarvestRandom => Action::Harvest,
                            Behavior::Heal(_) => Action::Heal,
                            _ => Action::Attack(entity),
                        }
                    } else {
                        ant_t.rotation = rotation;
                        Action::TargetedWalk(entity)
                    };
                }
            } else {
                // The target doesn't exist anymore
                ant.command = None;
                ant.behavior = AntCmp::base(&ant.kind).behavior;
                ant.action = Action::Idle;
            }
        }
    }
}

pub fn resolve_walk_action(
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    players: Res<Players>,
    mut map: ResMut<Map>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (mut ant_t, mut ant) in ant_q.iter_mut() {
        if let Action::Walk(target_loc) = ant.action {
            let player = players.get(ant.owner);

            let current_loc = map.get_loc(&ant_t.translation);
            if current_loc != target_loc {
                let speed = ant.speed
                    * game_settings.speed
                    * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED)
                    * if player.has_trait(&Trait::Haste) {
                        HASTE_SPEED_FACTOR
                    } else {
                        1.
                    };

                walk(
                    &mut ant_t,
                    &target_loc,
                    speed,
                    &mut map,
                    &game_settings,
                    &time,
                );
            } else {
                // Ant reached the target -> continue with default action
                ant.action = match ant.get_behavior() {
                    Behavior::Brood => {
                        if !player.queue.is_empty() {
                            Action::Brood(Timer::from_seconds(BROODING_TIME, TimerMode::Once))
                        } else {
                            Action::Idle
                        }
                    }
                    Behavior::Dig(_) | Behavior::DigRandom => {
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
    mut selected_q: Query<&mut Visibility, (With<SelectedCmp>, Without<AntHealthWrapperCmp>)>,
    mut leaf_q: Query<
        &mut Visibility,
        (
            With<LeafCarryCmp>,
            Without<SelectedCmp>,
            Without<AntHealthWrapperCmp>,
        ),
    >,
    mut nutrient_q: Query<
        &mut Visibility,
        (
            With<NutrientCarryCmp>,
            Without<LeafCarryCmp>,
            Without<SelectedCmp>,
            Without<AntHealthWrapperCmp>,
        ),
    >,
    children_q: Query<&Children>,
    selected_ants: Res<AntSelection>,
) {
    for (ant_e, ant_t, ant) in ant_q.iter() {
        for child in children_q.iter_descendants(ant_e) {
            if let Ok(mut selected_v) = selected_q.get_mut(child) {
                *selected_v = if selected_ants.0.contains(&ant_e) {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }

            if let Ok(mut leaf_v) = leaf_q.get_mut(child) {
                *leaf_v = if ant.carry.leaves >= ant.max_carry.leaves / 2. {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }

            if let Ok(mut nutrient_v) = nutrient_q.get_mut(child) {
                *nutrient_v = if ant.carry.nutrients >= ant.max_carry.nutrients / 2. {
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
                        egg.ant.size().x * 0.5 * egg_t.rotation.to_euler(EulerRot::ZXY).0.sin(),
                        egg.ant.size().y * 0.5 * egg_t.rotation.to_euler(EulerRot::ZXY).0.cos(),
                        0.1,
                    );

                    for child in children_q.iter_descendants(wrapper_e) {
                        if let Ok((mut health_t, mut health_s)) = health_q.get_mut(child) {
                            if let Some(size) = health_s.custom_size.as_mut() {
                                let full_size = egg.ant.size().x * 0.77;
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
    mut tile_q: Query<(Entity, &mut Sprite, &Tile)>,
    mut leaf_q: Query<&mut Sprite, (With<Leaf>, Without<Tile>)>,
    children_q: Query<&Children>,
    mut spawn_tile_ev: EventWriter<SpawnTileEv>,
    players: Res<Players>,
    mut map: ResMut<Map>,
) {
    let player = players.get(0);

    let mut visible_tiles = HashSet::new();

    // Calculate all tiles currently visible by the player
    ant_q
        .iter()
        .filter(|(_, _, _, a)| player.controls(a) && a.health > 0.)
        .for_each(|(_, ant_t, _, _)| {
            let current_tile = map.get_tile_from_coord(&ant_t.translation).unwrap();
            visible_tiles.extend(reveal_tiles(current_tile, &map, None, 0))
        });

    // Add tiles with 2 or more revealed neighbors to the list
    tile_q.iter().for_each(|(_, _, t)| {
        let visible_neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter(|(dx, dy)| {
                let nx = t.x as i32 + dx;
                let ny = t.y as i32 + dy;
                nx >= 0 && ny >= 0 && visible_tiles.contains(&(nx as u32, ny as u32))
            })
            .count();

        if visible_neighbors >= 2 {
            visible_tiles.insert((t.x, t.y));
        }
    });

    // Spawn new tiles if they are visible
    visible_tiles.iter().for_each(|(x, y)| {
        let tile = map.get_tile_mut(*x, *y).unwrap();

        tile.visible.insert(player.id);
        spawn_tile_ev.send(SpawnTileEv {
            tile: tile.clone(),
            pos: None,
        });
    });

    // Adjust the fog of war on the map
    tile_q.iter_mut().for_each(|(tile_e, mut sprite, tile)| {
        let color = if visible_tiles.contains(&(tile.x, tile.y)) {
            Color::WHITE
        } else {
            Color::srgba(1., 1., 1., 0.5)
        };

        sprite.color = color;

        // Update child (leaf) sprite color
        if let Ok(children) = children_q.get(tile_e) {
            for &child in children.iter() {
                if let Ok(mut leaf_s) = leaf_q.get_mut(child) {
                    leaf_s.color = color;
                }
            }
        }
    });

    // Show/hide enemies on the map
    for (_, ant_t, mut ant_v, ant) in ant_q.iter_mut() {
        if !player.controls(ant) && !ant.kind.is_ant() {
            if map
                .get_tile_from_coord(&ant_t.translation)
                .map_or(false, |tile| visible_tiles.contains(&(tile.x, tile.y)))
            {
                // The monster is visible, show it
                *ant_v = Visibility::Inherited;
            } else {
                // The monster is no longer visible, hide it
                *ant_v = Visibility::Hidden;
            }
        }
    }
}

pub fn queue_ants_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    players: Res<Players>,
    mut queue_ant_ev: EventWriter<QueueAntEv>,
) {
    for ant in Ant::iter().filter(|a| players.get(0).has_ant(a)) {
        if matches!(AntCmp::base(&ant).key, Some(key) if keyboard.just_pressed(key)) {
            queue_ant_ev.send(QueueAntEv { ant });
        }
    }
}
