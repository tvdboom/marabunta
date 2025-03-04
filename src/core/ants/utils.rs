use crate::core::ants::components::AntCmp;
use crate::core::map::loc::Loc;
use crate::core::map::map::Map;
use crate::core::resources::GameSettings;
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use std::f32::consts::PI;

pub fn walk(
    ant: &AntCmp,
    ant_t: &mut Transform,
    target_loc: &Loc,
    map: &mut ResMut<Map>,
    game_settings: &Res<GameSettings>,
    time: &Res<Time>,
) {
    let current_loc = map.get_loc(&ant_t.translation);
    let mut path = map.shortest_path(&current_loc, target_loc);
    if let Some(next_loc) = path.split_off(1).first() {
        // Calculate the distance vector to the next location
        let target_pos = Map::get_coord_from_loc(next_loc).extend(ant_t.translation.z);
        let d = -ant_t.translation + target_pos;

        let rotate = |r: Quat| {
            r.rotate_towards(
                Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                2. * game_settings.speed * time.delta_secs(),
            )
        };

        // Rotate towards the next location
        ant_t.rotation = rotate(ant_t.rotation);

        let speed = ant.speed * game_settings.speed * time.delta_secs();
        let next_pos = ant_t.translation + (ant_t.rotation * Vec3::Y).normalize() * speed;

        // If walking in a circle, increase the rotation
        // if (d.length() - (-next_pos + target_pos).length()).abs() < 0.5 * speed {
        //     ant_t.rotation = rotate(ant_t.rotation);
        //     ant_t.rotation = rotate(ant_t.rotation);
        //     ant_t.rotation = rotate(ant_t.rotation);
        // }

        let next_loc = map.get_loc(&next_pos);
        if next_loc == *target_loc
            || (map.is_walkable(&next_loc) || ant_t.rotation == rotate(ant_t.rotation))
        {
            ant_t.translation = next_pos;
        } else {
            // At a wall, rotate faster towards the next location
            ant_t.rotation = rotate(ant_t.rotation);
        }
    }
}
