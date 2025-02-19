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
    target_loc: &Loc,
    map: &Map,
    game_settings: &Res<GameSettings>,
    time: &Res<Time>,
) {
    let current_loc = map.get_loc(&ant_t.translation);
    let path = map.shortest_path(&current_loc, target_loc).split_off(1);

    if let Some(next_loc) = path.first() {
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

        let next_loc = map.get_loc(&next_pos);
        if map.is_walkable(&next_loc) || next_loc == *target_loc {
            ant_t.translation = next_pos;
        } else {
            // At a tunnel's wall, rotate faster towards the next location
            ant_t.rotation = ant_t.rotation.rotate_towards(
                Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                2. * game_settings.speed * time.delta_secs(),
            );
        }
    }
}
