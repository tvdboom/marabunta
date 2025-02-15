use std::f32::consts::PI;
use crate::core::ants::components::{Action, AnimationCmp, Ant, AntCmp};
use crate::core::map::components::Map;
use crate::core::resources::GameSettings;
use crate::utils::scale_duration;
use bevy::prelude::*;
use crate::core::assets::WorldAssets;

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
                atlas.index = atlas.index % animation.last_index + 1;
            }
        }
    }
}

pub fn spawn_ants(
    mut commands: Commands,
    map: Res<Map>,
    assets: Local<WorldAssets>,
) {
    let atlas = assets.atlas("black_ant_move");
    let ant = AntCmp::new(Ant::BlackAnt);
    commands.spawn((
        Sprite {
            image: atlas.image,
            texture_atlas: Some(atlas.texture),
            ..default()
        },
        Transform {
            translation: map.get_tile_coord(64).extend(3.),
            scale: Vec3::splat(ant.scale),
            ..default()
        },
        AnimationCmp {
            timer: Timer::from_seconds(ant.action.get_interval(), TimerMode::Repeating),
            last_index: atlas.last_index,
        },
        ant,
    ));
}

pub fn move_ants(
    mut ant_q: Query<(&mut AntCmp, &mut Transform)>,
    map: Res<Map>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (mut ant, mut ant_t) in ant_q.iter_mut() {
        let speed = ant.speed * game_settings.speed * time.delta_secs();

        match ant.action {
            Action::Wander(ref mut path) => {
                if let Some(path) = path {
                    if let Some(next_loc) = path.first() {
                        println!("Next loc: {:?}", next_loc);
                        let next_t = Map::get_coord(next_loc).extend(ant_t.translation.z);

                        let d = -ant_t.translation + next_t;

                        // Rotate towards the next location
                        ant_t.rotation = ant_t.rotation.rotate_towards(
                            Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                            game_settings.speed * time.delta_secs(),
                        );

                        let forward = (ant_t.rotation * Vec3::Y).normalize() * speed;
                        ant_t.translation += forward;

                        // If reached the next location, remove it from the path
                        if ant_t.translation.distance(next_t) < 8. {
                            path.remove(0);
                        }
                    } else {
                        // Reached the destination
                        ant.action = Action::Wander(None);
                    }
                } else {
                    // Determine new location to wander to
                    let a = map.random_walkable().expect("No walkable tiles.");
                    *path = map.shortest_path(Map::get_loc(&ant_t.translation), a);
                }
            }
        }
    }
}
