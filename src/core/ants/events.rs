use crate::core::ants::components::*;
use crate::core::assets::WorldAssets;
use crate::core::constants::{ANT_Z_SCORE, DEATH_TIME, EGG_Z_SCORE};
use crate::core::map::systems::MapCmp;
use crate::core::player::Player;
use crate::core::utils::{NoRotationChildCmp, NoRotationParentCmp};
use crate::utils::NameFromEnum;
use bevy::color::palettes::basic::{BLACK, LIME};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};
use uuid::Uuid;

#[derive(Event)]
pub struct QueueAntEv {
    pub ant: Ant,
}

#[derive(Event)]
pub struct SpawnEggEv {
    pub ant: Ant,
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
    pub attacker: Uuid,
    pub defender: Uuid,
}

pub fn queue_ant_event(
    mut queue_ant_ev: EventReader<QueueAntEv>,
    mut player: ResMut<Player>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    for ev in queue_ant_ev.read() {
        let ant_c = AntCmp::new(&ev.ant, player.id);

        if ant_c.key.is_some() {
            if player.food >= ant_c.price {
                player.food -= ant_c.price;
                player.queue.push_back(ant_c.kind);
                audio.play(assets.audio("button"));
            } else {
                audio.play(assets.audio("error"));
            }
        }
    }
}

pub fn spawn_egg_event(
    mut commands: Commands,
    mut spawn_egg_ev: EventReader<SpawnEggEv>,
    player: Res<Player>,
    assets: Local<WorldAssets>,
) {
    for SpawnEggEv { ant, transform } in spawn_egg_ev.read() {
        let ant_c = AntCmp::new(ant, player.id);
        let egg = Egg {
            id: Uuid::new_v4(),
            ant: ant_c.kind.clone(),
            owner: player.id,
            team: player.id,
            health: ant_c.max_health / 4.,
            max_health: ant_c.max_health / 4.,
            timer: Timer::from_seconds(ant_c.hatch_time, TimerMode::Once),
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
                    scale: Vec3::splat(0.5 * ant_c.scale),
                    ..default()
                },
                egg.clone(),
                NoRotationParentCmp,
                MapCmp,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Sprite {
                            color: Color::from(BLACK),
                            custom_size: Some(Vec2::new(egg.size().x * 0.8, egg.size().y * 0.1)),
                            ..default()
                        },
                        AntHealthWrapperCmp,
                        Visibility::Hidden,
                        NoRotationChildCmp,
                        MapCmp,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Sprite {
                                color: Color::from(LIME),
                                custom_size: Some(Vec2::new(
                                    egg.size().x * 0.77,
                                    egg.size().y * 0.08,
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
    mut player: ResMut<Player>,
    assets: Local<WorldAssets>,
) {
    for SpawnAntEv { ant, transform } in spawn_ant_ev.read() {
        let atlas = assets.atlas(&format!(
            "{}_{}",
            ant.kind.to_snake(),
            ant.animation().to_snake()
        ));

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
                ant.clone(),
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
                        Visibility::Hidden,
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
                    MapCmp,
                ));
            });

        if player.controls(ant) {
            player
                .colony
                .entry(ant.kind.clone())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }
}

pub fn despawn_ant_event(
    mut commands: Commands,
    ant_q: Query<&AntCmp>,
    mut despawn_ant_ev: EventReader<DespawnAntEv>,
    mut player: ResMut<Player>,
) {
    for DespawnAntEv { entity } in despawn_ant_ev.read() {
        if let Ok(ant) = ant_q.get(*entity) {
            if player.controls(ant) {
                player
                    .colony
                    .entry(ant.kind.clone())
                    .and_modify(|c| *c -= 1);
            }
        }

        commands.entity(*entity).despawn_recursive();
    }
}

pub fn damage_event(
    mut damage_ev: EventReader<DamageAntEv>,
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    mut egg_q: Query<(Entity, &mut Egg)>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
) {
    for DamageAntEv { attacker, defender } in damage_ev.read() {
        let damage = ant_q
            .iter_mut()
            .find(|(_, a)| a.id == *attacker)
            .unwrap()
            .1
            .damage;

        if let Some((mut defender_t, mut defender)) =
            ant_q.iter_mut().find(|(_, a)| a.id == *defender)
        {
            defender.health = (defender.health - damage).max(0.);
            if defender.health == 0. {
                defender.action = Action::Die(Timer::from_seconds(DEATH_TIME, TimerMode::Once));
                defender_t.translation.z = ANT_Z_SCORE;
            }
        } else if let Some((egg_e, mut egg)) = egg_q.iter_mut().find(|(_, a)| a.id == *defender) {
            egg.health = (egg.health - damage).max(0.);
            if egg.health == 0. {
                despawn_ant_ev.send(DespawnAntEv { entity: egg_e });
            }
        }
    }
}
