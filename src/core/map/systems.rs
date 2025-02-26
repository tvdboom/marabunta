use crate::core::ants::components::Ant;
use crate::core::ants::utils::spawn_ant;
use crate::core::assets::WorldAssets;
use crate::core::camera::{clamp_to_rect, MainCamera};
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::map::utils::spawn_tile;
use crate::core::player::Player;
use crate::core::resources::{GameMode, GameSettings};
use crate::core::states::GameState;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct MapCmp;

#[derive(Event)]
pub struct SwapTileEv(pub Tile);

pub fn create_map(game_settings: &GameSettings) -> Map {
    match game_settings.mode {
        GameMode::SinglePlayer => {
            // Insert base in the center of the map
            Map::from_base(
                UVec2::new(Map::MAP_SIZE.x / 2 - 2, Map::MAP_SIZE.y / 2 - 2),
                0,
            )
        }
        GameMode::MultiPlayer(n_players) => {
            let mut map = Map::new();

            // Insert bases at random locations
            let mut rng = rand::rng();
            let mut bases: Vec<UVec2> = Vec::new();

            while bases.len() < n_players {
                let candidate = UVec2 {
                    x: rng.random_range(5..Map::MAP_SIZE.x - 5),
                    y: rng.random_range(5..Map::MAP_SIZE.y - 5),
                };

                if bases
                    .iter()
                    .all(|pos| pos.as_vec2().distance(candidate.as_vec2()) >= 10.)
                {
                    bases.push(candidate);
                }
            }

            for (i, base) in bases.iter().enumerate() {
                map.insert_base(base, i);
            }

            map
        }
    }
}

pub fn draw_map(
    mut commands: Commands,
    mut camera_q: Query<(&mut Transform, &OrthographicProjection), With<MainCamera>>,
    map: Res<Map>,
    player: Res<Player>,
    game_state: Res<State<GameState>>,
    assets: Local<WorldAssets>,
) {
    for (i, tile) in map.world(player.id).iter_mut().enumerate() {
        let pos = Vec2::new(
            Map::WORLD_VIEW.min.x + Tile::SIZE * ((i as u32 % Map::WORLD_SIZE.x) as f32 + 0.5),
            Map::WORLD_VIEW.max.y - Tile::SIZE * ((i as u32 / Map::WORLD_SIZE.x) as f32 + 0.5),
        );

        spawn_tile(&mut commands, tile, pos, &assets);

        // Spawn queen
        if tile.texture_index == 9 {
            spawn_ant(&mut commands, Ant::BlackQueen, pos, player.id, &assets);

            // If in-game -> place camera on top of base
            if *game_state.get() == GameState::Game {
                let (mut camera_t, projection) = camera_q.single_mut();

                let position = camera_t.translation.truncate();
                let view_size = projection.area.max - projection.area.min;

                // Clamp camera position within bounds
                let target_pos = clamp_to_rect(position, view_size, Map::MAP_VIEW);
                camera_t.translation = target_pos.extend(camera_t.translation.z);
            }
        }
    }
}

pub fn swap_tile_event(
    mut commands: Commands,
    tile_q: Query<(Entity, &Tile)>,
    mut swap_tile_ev: EventReader<SwapTileEv>,
    assets: Local<WorldAssets>,
) {
    for ev in swap_tile_ev.read() {
        let new_t = ev.0.clone();
        commands
            .entity(
                tile_q
                    .iter()
                    .find(|(_, t)| t.x == new_t.x && t.y == new_t.y)
                    .unwrap()
                    .0,
            )
            .try_despawn_recursive();

        spawn_tile(
            &mut commands,
            &new_t,
            Map::get_coord_from_xy(new_t.x, new_t.y),
            &assets,
        );
    }
}

pub fn update_map(
    tile_q: Query<&Tile>,
    mut swap_tile_ev: EventWriter<SwapTileEv>,
    player: Res<Player>,
) {
    tile_q
        .iter()
        .filter(|t| t.visible.contains(&player.id) && t.is_soil())
        .for_each(|tile| {
            swap_tile_ev.send(SwapTileEv(tile.clone()));
        });
}
