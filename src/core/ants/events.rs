use crate::core::ants::components::{
    AnimationCmp, Ant, AntCmp, AntHealthCmp, AntHealthWrapperCmp, LeafCarryCmp,
};
use crate::core::assets::WorldAssets;
use crate::core::constants::ANT_Z_SCORE;
use crate::core::map::systems::MapCmp;
use crate::core::player::Player;
use crate::core::utils::{NoRotationChildCmp, NoRotationParentCmp};
use crate::utils::NameFromEnum;
use bevy::color::palettes::basic::{BLACK, LIME};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};

#[derive(Event)]
pub struct QueueAntEv {
    pub ant: Ant,
}

#[derive(Event)]
pub struct SpawnAntEv {
    pub ant: AntCmp,
    pub transform: Transform,
}

#[derive(Event)]
pub struct DespawnAntEv {
    pub ant: AntCmp,
    pub entity: Entity,
}

pub fn queue_ants(
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
                player.queue.push(ant_c.kind);
                audio.play(assets.audio("button"));
            } else {
                audio.play(assets.audio("error"));
            }
        }
    }
}

pub fn spawn_ants(
    mut commands: Commands,
    mut spawn_ant_ev: EventReader<SpawnAntEv>,
    mut player: ResMut<Player>,
    assets: Local<WorldAssets>,
) {
    for SpawnAntEv { ant, transform } in spawn_ant_ev.read() {
        let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));

        let animation = ant.action.animation();
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
                    animation: animation.clone(),
                    timer: Timer::from_seconds(ant.kind.interval(&animation), TimerMode::Repeating),
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

pub fn despawn_ants(
    mut commands: Commands,
    mut despawn_ant_ev: EventReader<DespawnAntEv>,
    mut player: ResMut<Player>,
) {
    for DespawnAntEv { ant, entity } in despawn_ant_ev.read() {
        commands.entity(*entity).despawn_recursive();

        if player.controls(ant) {
            player
                .colony
                .entry(ant.kind.clone())
                .and_modify(|c| *c -= 1);
        }
    }
}
