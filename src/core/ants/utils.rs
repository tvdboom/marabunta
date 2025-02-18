use crate::core::ants::components::AntCmp;
use crate::core::map::components::{Loc, Map};
use crate::core::resources::GameSettings;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Res, Time, Transform};
use std::f32::consts::PI;

/// Returns whether the ant is at the target location
pub fn walk(
    ant: &AntCmp,
    ant_t: &mut Transform,
    target_loc: &mut Loc,
    map: &Map,
    game_settings: &Res<GameSettings>,
    time: &Res<Time>,
) -> bool {
    let current_loc = Map::get_loc(&ant_t.translation);
    println!("Current loc: {:?}", current_loc);
    println!("Target loc: {:?}", target_loc);

    if current_loc != *target_loc {
        let path = map.shortest_path(current_loc, *target_loc).split_off(1);
        println!("Path: {:?}", path);

        if let Some(next_loc) = path.first() {
            println!("Next loc: {:?}", next_loc);
            // Calculate the distance vector to the next location
            let d = -ant_t.translation + Map::get_coord(next_loc).extend(ant_t.translation.z);

            // Rotate towards the next location
            ant_t.rotation = ant_t.rotation.rotate_towards(
                Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                game_settings.speed * time.delta_secs(),
            );

            let next_pos = ant_t.translation
                + (ant_t.rotation * Vec3::Y).normalize()
                    * ant.speed
                    * game_settings.speed
                    * time.delta_secs();

            if map.is_walkable(&Map::get_loc(&next_pos)) {
                ant_t.translation = next_pos;
            } else {
                // At a tunnel's wall, rotate faster towards the next location
                ant_t.rotation = ant_t.rotation.rotate_towards(
                    Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                    2. * game_settings.speed * time.delta_secs(),
                );
            }
        }

        false
    } else {
        true
    }
}
