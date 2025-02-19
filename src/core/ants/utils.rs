use crate::core::ants::components::{AnimationCmp, Ant, AntCmp};
use crate::core::map::components::{Loc, Map};
use crate::core::resources::GameSettings;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{default, Local, Res, Sprite, Time, Timer, TimerMode, Transform};
use std::f32::consts::PI;
use rand::Rng;
use crate::core::assets::WorldAssets;
use crate::core::constants::ANT_Z_SCORE;
use crate::core::map::systems::MapCmp;
use crate::utils::NameFromEnum;

pub fn spawn_ant(
    kind: Ant,
    pos: Vec2,
    assets: &Local<WorldAssets>,
) -> (Sprite, Transform, AnimationCmp, AntCmp, MapCmp) {
    let ant = AntCmp::new(kind);

    let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));

    (
        Sprite {
            image: atlas.image,
            texture_atlas: Some(atlas.texture),
            ..default()
        },
        Transform {
            translation: pos.extend(ANT_Z_SCORE + ant.z_score),
            rotation: Quat::from_rotation_z(rand::rng().random_range(0.0..2. * PI)),
            scale: Vec3::splat(ant.scale),
            ..default()
        },
        AnimationCmp {
            timer: Timer::from_seconds(ant.action.interval(), TimerMode::Repeating),
            last_index: atlas.last_index,
            action: ant.action.clone(),
        },
        ant,
        MapCmp,
    )
}

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
        if path.len() == 1 || next_loc == *target_loc || map.is_walkable(&next_loc) {
            ant_t.translation = next_pos;
        } else {
            // At a wall, rotate faster towards the next location
            ant_t.rotation = ant_t.rotation.rotate_towards(
                Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                2. * game_settings.speed * time.delta_secs(),
            );
        }
    }
}
