use crate::core::ants::components::{AnimationCmp, Ant, AntCmp};
use crate::core::assets::WorldAssets;
use crate::core::map::components::Map;
use crate::core::map::tile::Tile;
use crate::core::states::{GameState, PauseState};
use bevy::prelude::*;
use rand::{rng, Rng};

#[derive(Component)]
pub struct MapCmp;

#[derive(Component)]
pub struct StoneCmp;

pub fn draw_start_map(mut commands: Commands, assets: Local<WorldAssets>) {
    let mut map = Map::new();
    map.insert_base(UVec2::new(
        Map::MAP_SIZE.x / 2 - 16,
        Map::MAP_SIZE.y / 2 - 6,
    ));

    for (y, col) in map.world().iter().enumerate() {
        for (x, tile) in col.iter().enumerate() {
            let texture = assets.texture("tiles");
            commands.spawn((
                Sprite {
                    image: texture.image,
                    custom_size: Some(Vec2::splat(Tile::SIZE)),
                    texture_atlas: Some(TextureAtlas {
                        layout: texture.layout,
                        index: tile.texture_index,
                    }),
                    ..default()
                },
                Transform {
                    translation: Map::get_world_coord(x, y).extend(0.),
                    rotation: Quat::from_rotation_z((tile.rotation as f32).to_radians()),
                    ..default()
                },
                tile.clone(),
                MapCmp,
            ));

            // Add random stones for decoration
            if Tile::SOIL.contains(&tile.texture_index) && rand::random::<f32>() > 0.9 {
                commands.spawn((
                    Sprite {
                        image: assets.image(&format!("stone{}", rng().random_range(1..=18))),
                        ..default()
                    },
                    Transform {
                        translation: Map::get_world_coord(x, y).extend(1.),
                        rotation: Quat::from_rotation_z(
                            rng().random_range(0.0_f32..360.).to_radians(),
                        ),
                        scale: Vec3::splat(rng().random_range(0.1..0.2)),
                        ..default()
                    },
                    StoneCmp,
                    MapCmp,
                ));
            }

            // Spawn queen at hole
            if tile.texture_index == 64 {
                let atlas = assets.atlas("black_queen_move");
                let ant = AntCmp::new(Ant::BlackQueen);
                commands.spawn((
                    Sprite {
                        image: atlas.image,
                        texture_atlas: Some(atlas.texture),
                        ..default()
                    },
                    Transform {
                        translation: Map::get_world_coord(x, y).extend(3.),
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
        }
    }

    commands.insert_resource(map);
}

pub fn toggle_pause_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if *game_state.get() == GameState::Game {
        if keyboard.just_pressed(KeyCode::Space) {
            match pause_state.get() {
                PauseState::Running => next_pause_state.set(PauseState::Paused),
                PauseState::Paused => next_pause_state.set(PauseState::Running),
            }
        }
    }
}
