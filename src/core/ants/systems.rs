use std::f32::consts::PI;
use crate::core::ants::components::{AnimationCmp, AntCmp, Movement};
use crate::core::map::components::Map;
use crate::core::resources::GameSettings;
use crate::utils::scale_duration;
use bevy::prelude::*;

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

pub fn move_ants(
    mut ant_q: Query<(&mut AntCmp, &mut Transform)>,
    map: Res<Map>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (mut ant, mut ant_t) in ant_q.iter_mut() {
        let speed = ant.speed * game_settings.speed * time.delta_secs();

        match ant.movement {
            Movement::Wander(ref mut path) => {
                if let Some(path) = path {
                    if let Some(next_loc) = path.first() {
                        let next_t = Map::get_coord(next_loc).extend(ant_t.translation.z);

                        let d = -ant_t.translation + next_t;

                        // Rotate towards the next location
                        ant_t.rotation = ant_t.rotation.rotate_towards(
                            Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                            game_settings.speed * time.delta_secs(),
                        );

                        ant_t.translation += d.normalize() * speed;

                        // If reached the next location, remove it from the path
                        if ant_t.translation.distance(next_t) < 2. {
                            path.remove(0);
                        }
                    } else {
                        // Reached the destination
                        ant.movement = Movement::Wander(None);
                    }
                } else {
                    // Determine new location to wander to
                    let a = map.random_walkable().expect("No walkable tiles.");
                    println!("Wandering to {:?}", a);
                    *path = map.shortest_path(Map::get_loc(&ant_t.translation), a);
                }
            }
        }
    }
}
