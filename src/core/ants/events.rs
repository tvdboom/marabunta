use crate::core::ants::components::{AnimationCmp, AntCmp, AntHealth, AntHealthWrapper};
use crate::core::assets::WorldAssets;
use crate::core::constants::ANT_Z_SCORE;
use crate::core::map::systems::MapCmp;
use crate::utils::NameFromEnum;
use bevy::color::palettes::basic::{BLACK, LIME};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use crate::core::resources::Population;

#[derive(Event)]
pub struct SpawnAntEv {
    pub ant: AntCmp,
    pub transform: Transform,
}

#[derive(Event)]
pub struct DespawnAntEv {
    pub entity: Entity,
}

pub fn spawn_ants(
    mut commands: Commands,
    mut spawn_ant_ev: EventReader<SpawnAntEv>,
    assets: Local<WorldAssets>,
) {
    for SpawnAntEv { ant, transform } in spawn_ant_ev.read() {
        let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));

        let id = commands
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
                MapCmp,
            ))
            .id();

        // Spawn health bar
        commands
            .spawn((
                Sprite {
                    color: Color::from(BLACK),
                    custom_size: Some(Vec2::new(ant.size().x * 0.8, ant.size().y * 0.1)),
                    ..default()
                },
                AntHealthWrapper(id),
                Visibility::Hidden,
                MapCmp,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Sprite {
                        color: Color::from(LIME),
                        custom_size: Some(Vec2::new(ant.size().x * 0.77, ant.size().y * 0.08)),
                        ..default()
                    },
                    Transform::from_xyz(0., 0., 0.1),
                    AntHealth,
                ));
            });
    }
}

pub fn despawn_ants(
    mut commands: Commands,
    ant_q: Query<&AntCmp>,
    wrapper_q: Query<(Entity, &AntHealthWrapper)>,
    mut despawn_ant_ev: EventReader<DespawnAntEv>,
    mut population: ResMut<Population>,
) {
    for DespawnAntEv { entity } in despawn_ant_ev.read() {
        population.0.remove(&ant_q.get(*entity).unwrap().id);

        commands
            .entity(wrapper_q.iter().find(|(_, w)| w.0 == *entity).unwrap().0)
            .despawn_recursive();
        commands.entity(*entity).despawn();
    }
}
