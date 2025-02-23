use crate::core::ants::components::{AnimationCmp, Ant, AntCmp, AntHealth, AntHealthWrapper};
use crate::core::assets::WorldAssets;
use crate::core::constants::ANT_Z_SCORE;
use crate::core::map::loc::Loc;
use crate::core::map::map::Map;
use crate::core::map::systems::MapCmp;
use crate::core::resources::GameSettings;
use crate::utils::NameFromEnum;
use bevy::color::palettes::css::{BLACK, LIME};
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

pub fn spawn_ant(commands: &mut Commands, kind: Ant, pos: Vec2, assets: &Local<WorldAssets>) {
    let ant = AntCmp::new(kind);

    let atlas = assets.atlas(&format!("{}_{}", ant.kind.to_snake(), ant.action.to_name()));

    let id = commands
        .spawn((
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
                action: ant.action.clone(),
                timer: Timer::from_seconds(ant.action.interval(), TimerMode::Repeating),
                last_index: atlas.last_index,
            },
            ant.clone(),
            MapCmp,
        ))
        .id();

    // Spawn health bars
    commands
        .spawn((
            Sprite {
                color: Color::from(BLACK),
                custom_size: Some(Vec2::new(ant.size().x * 0.8, ant.size().y * 0.1)),
                ..default()
            },
            AntHealthWrapper(id),
            MapCmp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: Color::from(LIME),
                    custom_size: Some(Vec2::new(ant.size().x * 0.77, ant.size().y * 0.08)),
                    ..default()
                },
                Transform::from_xyz(0., 0., 0.1),
                AntHealth,
            ));
        });
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
    println!("{:?}", map.get_tile(&current_loc));
    let path = map.shortest_path(&current_loc, target_loc).split_off(1);

    if let Some(next_loc) = path.first() {
        // Calculate the distance vector to the next location
        let target_pos = Map::get_coord(next_loc).extend(ant_t.translation.z);
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
        if (d.length() - (-next_pos + target_pos).length()).abs() < 0.1 * speed {
            ant_t.rotation = rotate(ant_t.rotation);
        }

        let next_loc = map.get_loc(&next_pos);
        if path.len() == 1
            || next_loc == *target_loc
            || (map.is_walkable(&next_loc) || ant_t.rotation == rotate(ant_t.rotation))
        {
            ant_t.translation = next_pos;
        } else {
            // At a wall, rotate faster towards the next location
            ant_t.rotation = rotate(ant_t.rotation);
        }
    }
}
