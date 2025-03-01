use crate::core::ants::components::{AnimationCmp, AntCmp, AntHealthCmp, AntHealthWrapperCmp};
use crate::core::assets::WorldAssets;
use crate::core::constants::ANT_Z_SCORE;
use crate::core::map::systems::MapCmp;
use crate::core::utils::{NoRotationChildCmp, NoRotationParentCmp};
use crate::utils::NameFromEnum;
use bevy::color::palettes::basic::{BLACK, LIME};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;

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
pub struct CarryLeafEv {
    pub entity: Entity,
}

#[derive(Event)]
pub struct UncarryLeafEv {
    pub entity: Entity,
}

pub fn spawn_ants(
    mut commands: Commands,
    mut spawn_ant_ev: EventReader<SpawnAntEv>,
    assets: Local<WorldAssets>,
) {
    for SpawnAntEv { ant, transform } in spawn_ant_ev.read() {
        let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));

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
                    action: ant.action.clone(),
                    timer: Timer::from_seconds(ant.action.interval(), TimerMode::Repeating),
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
            });
    }
}

pub fn despawn_ants(mut commands: Commands, mut despawn_ant_ev: EventReader<DespawnAntEv>) {
    for DespawnAntEv { entity } in despawn_ant_ev.read() {
        commands.entity(*entity).despawn_recursive();
    }
}
