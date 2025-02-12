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
    for (mut ant, mut transform) in ant_q.iter_mut() {
        let current_loc = ant.loc;
        match ant.movement {
            Movement::Wander(ref mut path) => {
                if path.is_none() {
                    // Determine new location to wander to
                    *path = map.shortest_path(
                        current_loc,
                        map.random_walkable().expect("No walkable tiles."),
                    );
                }

                if let Some(path) = path {}
            }
        }
    }
}
