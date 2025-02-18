use crate::core::ants::components::{Action, Animation, AnimationCmp, Ant, AntCmp};
use crate::core::ants::utils::walk;
use crate::core::assets::WorldAssets;
use crate::core::map::components::Map;
use crate::core::map::systems::MapCmp;
use crate::core::resources::GameSettings;
use crate::utils::{scale_duration, NameFromEnum};
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

#[derive(Event)]
pub struct ChangeAnimation {
    pub entity: Entity,
    pub animation: Animation,
}

pub fn spawn_ant(commands: &mut Commands, kind: Ant, pos: Vec2, assets: &Local<WorldAssets>) {
    let ant = AntCmp::new(kind);

    let atlas = assets.atlas(&format!("{}_move", ant.name.to_snake()));
    commands.spawn((
        Sprite {
            image: atlas.image,
            texture_atlas: Some(atlas.texture),
            ..default()
        },
        Transform {
            translation: pos.extend(3. + ant.z_score),
            rotation: Quat::from_rotation_z(rand::rng().random_range(0.0..2. * PI)),
            scale: Vec3::splat(ant.scale),
            ..default()
        },
        AnimationCmp {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            last_index: atlas.last_index,
        },
        ant,
        MapCmp,
    ));
}

pub fn animate_ants(
    mut ant_q: Query<(&mut Sprite, &mut AnimationCmp), With<AntCmp>>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation) in ant_q.iter_mut() {
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
    }
}

pub fn resolve_action_ants(
    mut ant_q: Query<(Entity, &mut AntCmp, &mut Transform)>,
    mut action_ev: EventWriter<ChangeAnimation>,
    map: Res<Map>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (ant_e, mut ant, mut ant_t) in ant_q.iter_mut() {
        match ant.action {
            Action::Idle => {
            }
            Action::Walk(ref mut loc) => {
                if let Some(l) = loc {
                    if walk(&ant, &mut ant_t, l, &map, &game_settings, &time) {
                        *loc = None; // Reached the target
                    }
                } else {
                    // Determine new location to wander to
                    *loc = Some(map.random_walk_loc().expect("No location to walk."));
                }
            }
            Action::Dig => {
                if let Some(l) = loc {
                    if walk(&ant, &mut ant_t, l, &map, &game_settings, &time)
                        && ant.animation != Animation::Dig
                    {
                        // Start digging
                        println!("Digging at {:?}", l);
                        action_ev.send(ChangeAnimation {
                            entity: ant_e,
                            animation: Animation::Dig,
                        });
                    }
                } else {
                    // Determine new location to dig
                    *loc = Some(map.random_dig_loc().expect("No location to dig."));
                }
            }
        }
    }
}

pub fn change_ant_animation(
    mut action_ev: EventReader<ChangeAnimation>,
    mut ant_q: Query<(&mut Sprite, &mut AnimationCmp, &mut AntCmp)>,
    assets: Local<WorldAssets>,
) {
    for ev in action_ev.read() {
        let (mut ant_s, mut animation, mut ant) = ant_q.get_mut(ev.entity).unwrap();

        ant.animation = ev.animation.clone();

        let atlas = assets.atlas(&format!(
            "{}_{}",
            ant.name.to_snake(),
            ev.animation.to_name()
        ));
        ant_s.image = atlas.image;
        ant_s.texture_atlas = Some(atlas.texture);
        animation.last_index = atlas.last_index;
    }
}
