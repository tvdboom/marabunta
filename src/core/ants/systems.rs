use crate::core::ants::components::{AnimationCmp, AntCmp, Movement};
use crate::core::map::components::Map;
use crate::core::resources::GameSettings;
use crate::utils::scale_duration;
use bevy::prelude::*;
use std::f32::consts::PI;

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
                    // println!("{:?}", path);
                    if let Some(next_loc) = path.first() {
                        // println!("Next loc: {:?}", next_loc);
                        let next_t = Map::get_coord(next_loc).extend(ant_t.translation.z);

                        // println!("Next t: {:?}", next_t);
                        let d = -ant_t.translation + next_t;

                        // Rotate towards the next location
                        ant_t.rotation = ant_t.rotation.rotate_towards(
                            Quat::from_rotation_z(d.y.atan2(d.x) - PI),
                            game_settings.speed * time.delta_secs(),
                        );
                        // let target_rotation = Quat::from_rotation_z(d.y.atan2(d.x));
                        // ant_t.rotation = ant_t.rotation.slerp(target_rotation, game_settings.speed * time.delta_secs());

                        let d_pos = ant_t.rotation.mul_vec3(Vec3::X).normalize() * speed;
                        ant_t.translation += d_pos;

                        // If reached the next location, remove it from the path
                        if ant_t.translation.distance(next_t) < 0.1 {
                            path.remove(0);
                        }
                    } else {
                        // Reached the destination
                        ant.movement = Movement::Wander(None);
                    }
                } else {
                    // Determine new location to wander to
                    let a = map.random_walkable().expect("No walkable tiles.");
                    println!("New loc: {:?}", a);
                    println!("Current loc: {:?}", Map::get_loc(&ant_t.translation));
                    println!(
                        "neighbors: {:?}",
                        map.get_neighbors(Map::get_loc(&ant_t.translation))
                    );
                    *path = map.shortest_path(Map::get_loc(&ant_t.translation), a);
                    println!("{:?}", path);
                }
            }
        }
    }
}
