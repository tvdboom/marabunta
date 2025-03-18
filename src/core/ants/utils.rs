use crate::core::ants::components::AntCmp;
use crate::core::game_settings::GameSettings;
use crate::core::map::loc::Loc;
use crate::core::map::map::Map;
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use std::f32::consts::PI;

pub fn transform_ant(transform: &mut Transform, ant: &mut AntCmp, new_ant: &AntCmp) {
    transform.scale = Vec3::splat(new_ant.scale);

    let mut new_ant = new_ant.clone();
    new_ant.id = ant.id;

    // Keep the same health ratio as it currently has
    new_ant.health = (ant.health / ant.max_health) * new_ant.max_health;

    *ant = new_ant;
}

pub fn walk(
    ant_t: &mut Transform,
    target_loc: &Loc,
    speed: f32,
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
                // Rotate faster when closer to the target to avoid walking in circles
                (2. + 3. / (1. + ((d.length() - 8.).max(0.).exp())))
                    * game_settings.speed
                    * time.delta_secs(),
            )
        };

        // Rotate towards the next location
        ant_t.rotation = rotate(ant_t.rotation);

        let next_pos = ant_t.translation + (ant_t.rotation * Vec3::Y).normalize() * speed;

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
