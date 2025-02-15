use crate::core::ants::components::{Action, AnimationCmp, Ant, AntCmp};
use crate::core::assets::WorldAssets;
use crate::core::map::components::Map;
use crate::core::resources::GameSettings;
use crate::utils::{scale_duration, NameFromEnum};
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

pub fn spawn_ant(commands: &mut Commands, kind: Ant, pos: Vec2, assets: &Local<WorldAssets>) {
    let atlas = assets.atlas(&format!("{}_move", kind.to_snake()));
    let ant = AntCmp::new(kind);
    commands.spawn((
        Sprite {
            image: atlas.image,
            texture_atlas: Some(atlas.texture),
            ..default()
        },
        Transform {
            translation: pos.extend(3.),
            rotation: Quat::from_rotation_z(rand::rng().random_range(0.0..2. * PI)),
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

        match ant.action {
            Action::Wander(ref mut path) => {
                if let Some(path) = path {
                    if let Some(next_loc) = path.first() {
                        let next_t = Map::get_coord(next_loc).extend(ant_t.translation.z);

                        let d = -ant_t.translation + next_t;

                        // Rotate towards the next location
                        ant_t.rotation = ant_t.rotation.rotate_towards(
                            Quat::from_rotation_z(d.y.atan2(d.x) - PI * 0.5),
                            2. * game_settings.speed * time.delta_secs(),
                        );

                        // Check if next position is walkable, else only turn
                        let next_pos =
                            ant_t.translation + (ant_t.rotation * Vec3::Y).normalize() * speed;

                        println!("{:?}", Map::get_loc(&next_pos));
                        println!("{}", map.is_walkable(&Map::get_loc(&next_pos)));
                        if map.is_walkable(&Map::get_loc(&next_pos)) {
                            ant_t.translation = next_pos;
                        }

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
                    let loc = map.random_walkable().expect("No walkable tiles.");
                    *path = map.shortest_path(Map::get_loc(&ant_t.translation), loc);
                }
            }
        }
    }
}
