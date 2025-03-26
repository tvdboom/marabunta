use crate::core::ants::components::*;
use crate::core::ants::selection::select_ant_on_click;
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioEv;
use crate::core::constants::*;
use crate::core::game_settings::GameSettings;
use crate::core::map::systems::MapCmp;
use crate::core::menu::settings::FogOfWar;
use crate::core::player::Players;
use crate::core::states::GameState;
use crate::core::traits::Trait;
use crate::core::utils::{NoRotationChildCmp, NoRotationParentCmp};
use bevy::color::palettes::basic::{BLACK, LIME};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;

#[derive(Event)]
pub struct QueueAntEv {
    pub ant: Ant,
}

#[derive(Event)]
pub struct SpawnEggEv {
    pub ant: AntCmp,
    pub transform: Transform,
}

#[derive(Event)]
pub struct SpawnAntEv {
    pub ant: AntCmp,
    pub transform: Transform,
}

#[derive(Event)]
pub struct DespawnAntEv {
    pub entity: Entity,
}

#[derive(Event)]
pub struct DamageAntEv {
    pub attacker: Entity,
    pub defender: Entity,
}

pub fn queue_ant_event(
    mut queue_ant_ev: EventReader<QueueAntEv>,
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    mut players: ResMut<Players>,
) {
    let player = players.get_mut(0);

    for ev in queue_ant_ev.read() {
        let ant_c = AntCmp::base(&ev.ant);

        if ant_c.key.is_some() {
            let price = ant_c.price
                * if player.has_trait(&Trait::MegaColony) {
                    ANT_PRICE_FACTOR
                } else {
                    1.
                };

            if player.resources >= price && player.queue.len() < MAX_QUEUE_LENGTH {
                player.resources -= &price;
                player.queue.push_back(ant_c.kind);
                play_audio_ev.send(PlayAudioEv::new("button"));
            } else {
                play_audio_ev.send(PlayAudioEv::new("error"));
            }
        }
    }
}

pub fn spawn_egg_event(
    mut commands: Commands,
    mut spawn_egg_ev: EventReader<SpawnEggEv>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    assets: Local<WorldAssets>,
) {
    for SpawnEggEv { ant, transform } in spawn_egg_ev.read() {
        let player = players.get(ant.team);

        let health_factor = if player.has_trait(&Trait::Breeding) {
            2. * EGG_HEALTH_FACTOR
        } else {
            EGG_HEALTH_FACTOR
        };

        let egg = Egg {
            ant: ant.clone(),
            team: player.id,
            health: ant.max_health / health_factor,
            max_health: ant.max_health / health_factor,
            timer: Timer::from_seconds(ant.hatch_time, TimerMode::Once),
        };

        commands
            .spawn((
                Sprite {
                    image: assets.image("larva2"),
                    ..default()
                },
                Transform {
                    translation: transform.translation.truncate().extend(EGG_Z_SCORE),
                    rotation: transform.rotation,
                    scale: Vec3::splat(0.5 * ant.scale),
                    ..default()
                },
                egg.clone(),
                TeamCmp(egg.team),
                NoRotationParentCmp,
                MapCmp,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Sprite {
                            color: Color::from(BLACK),
                            custom_size: Some(Vec2::new(ant.size().x * 0.8, ant.size().y * 0.1)),
                            ..default()
                        },
                        AntHealthWrapperCmp,
                        if game_settings.fog_of_war == FogOfWar::None || egg.team == 0 {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        },
                        NoRotationChildCmp,
                        MapCmp,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Sprite {
                                color: Color::from(LIME),
                                custom_size: Some(Vec2::new(
                                    ant.size().x * 0.77,
                                    ant.size().y * 0.08,
                                )),
                                ..default()
                            },
                            Transform::from_xyz(0., 0., 0.1),
                            AntHealthCmp,
                        ));
                    });
            });
    }
}

pub fn spawn_ant_event(
    mut commands: Commands,
    mut spawn_ant_ev: EventReader<SpawnAntEv>,
    game_settings: Res<GameSettings>,
    mut players: ResMut<Players>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Local<WorldAssets>,
) {
    for SpawnAntEv { ant, transform } in spawn_ant_ev.read() {
        let atlas = assets.atlas(&ant.atlas(&ant.animation()));
        commands
            .spawn((
                Sprite {
                    image: atlas.image,
                    texture_atlas: Some(atlas.texture),
                    ..default()
                },
                Transform {
                    translation: transform
                        .translation
                        .truncate()
                        .extend(ANT_Z_SCORE + ant.z_score),
                    rotation: transform.rotation,
                    scale: Vec3::splat(ant.scale),
                    ..default()
                },
                AnimationCmp {
                    animation: ant.animation(),
                    timer: Timer::from_seconds(
                        ant.kind.interval(&ant.animation()),
                        TimerMode::Repeating,
                    ),
                    last_index: atlas.last_index,
                },
                TeamCmp(ant.team),
                ant.clone(),
                if game_settings.fog_of_war == FogOfWar::None || ant.team == 0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                NoRotationParentCmp,
                MapCmp,
            ))
            .observe(select_ant_on_click)
            .with_children(|parent| {
                parent
                    .spawn((
                        Sprite {
                            color: Color::from(BLACK),
                            custom_size: Some(Vec2::new(ant.size().x * 0.8, ant.size().y * 0.1)),
                            ..default()
                        },
                        AntHealthWrapperCmp,
                        Visibility::Hidden,
                        NoRotationChildCmp,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Sprite {
                                color: Color::from(LIME),
                                custom_size: Some(Vec2::new(
                                    ant.size().x * 0.77,
                                    ant.size().y * 0.08,
                                )),
                                ..default()
                            },
                            Transform::from_xyz(0., 0., 0.1),
                            AntHealthCmp,
                        ));
                    });

                let r = 0.4 * ant.size().min_element();
                parent.spawn((
                    Mesh2d(meshes.add(Annulus::new(r, 1.1 * r))),
                    MeshMaterial2d(
                        materials.add(ColorMaterial::from(Color::srgba(0., 0., 0., 0.8))),
                    ),
                    Transform::from_translation(Vec3::new(0., 0., -0.1)),
                    SelectedCmp,
                    Visibility::Hidden,
                ));

                parent.spawn((
                    Sprite {
                        image: assets.image("leaf2"),
                        ..default()
                    },
                    Transform {
                        translation: Vec3::new(0., 5., 0.1),
                        scale: Vec3::splat(3.),
                        ..default()
                    },
                    LeafCarryCmp,
                    Visibility::Hidden,
                ));

                parent.spawn((
                    Sprite {
                        image: assets.image("blood"),
                        ..default()
                    },
                    Transform {
                        translation: Vec3::new(0., -75., 0.2),
                        scale: Vec3::splat(0.1),
                        ..default()
                    },
                    NutrientCarryCmp,
                    Visibility::Hidden,
                ));
            });

        players
            .get_mut(ant.team)
            .colony
            .entry(ant.kind.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }
}

pub fn despawn_ant_event(
    mut commands: Commands,
    mut despawn_ant_ev: EventReader<DespawnAntEv>,
    mut next_game_state: ResMut<NextState<GameState>>,
    players: Res<Players>,
) {
    for DespawnAntEv { entity } in despawn_ant_ev.read() {
        if players.get(0).colony[&Ant::Queen] == 0 {
            next_game_state.set(GameState::GameOver);
        } else {
            commands.entity(*entity).despawn_recursive();
        }
    }
}

pub fn damage_event(
    mut commands: Commands,
    mut damage_ev: EventReader<DamageAntEv>,
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    mut egg_q: Query<(Entity, &mut Egg)>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
    mut players: ResMut<Players>,
) {
    for DamageAntEv { attacker, defender } in damage_ev.read() {
        let attacker = ant_q.get(*attacker).unwrap().1;
        let player_a = players.get(attacker.team);
        let damage = attacker.damage;

        if let Ok((mut ant_t, mut ant_c)) = ant_q.get_mut(*defender) {
            // Apply extra bonus factors against monsters
            let damage = damage
                * if (ant_c.kind.is_scorpion() && player_a.has_trait(&Trait::ScorpionKiller))
                    || (ant_c.kind == Ant::Wasp && player_a.has_trait(&Trait::WaspKiller))
                    || (ant_c.kind.is_termite() && player_a.has_trait(&Trait::TermiteKiller))
                {
                    2.
                } else {
                    1.
                };

            ant_c.health = (ant_c.health - damage).max(0.);

            if ant_c.health == 0. && !matches!(ant_c.action, Action::Die(_)) {
                let player_d = players.get_mut(ant_c.team);

                commands.entity(*defender).insert(Corpse);

                ant_c.action = Action::Die(Timer::from_seconds(
                    DEATH_TIME
                        * if player_d.has_trait(&Trait::Corpses) {
                            2.
                        } else {
                            1.
                        },
                    TimerMode::Once,
                ));

                ant_t.translation.z = ANT_Z_SCORE;

                if ant_c.team == player_d.id {
                    player_d.colony.entry(ant_c.kind.clone()).and_modify(|c| {
                        *c = c.saturating_sub(1);
                    });

                    // If the queen died, the player lost the game
                    if player_d.id == 0 && player_d.colony[&Ant::Queen] == 0 {
                        play_audio_ev.send(PlayAudioEv::new("game-over"));
                    }
                }
            }
        } else if let Ok((egg_e, mut egg)) = egg_q.get_mut(*defender) {
            egg.health = (egg.health - damage).max(0.);
            if egg.health == 0. {
                despawn_ant_ev.send(DespawnAntEv { entity: egg_e });
            }
        }
    }
}
