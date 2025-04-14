use crate::core::ants::components::TeamCmp;
use crate::core::ants::selection::{select_leaf_on_click, select_loc_on_click};
use crate::core::assets::WorldAssets;
use crate::core::constants::{LEAF_TEAM, NON_MAP_ID, NO_VISION_COLOR, TILE_Z_SCORE};
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Tile;
use crate::core::menu::settings::{Background, FogOfWar};
use crate::core::network::{ClientMessage, ClientSendMessage, ServerMessage, ServerSendMessage};
use crate::core::player::Players;
use crate::core::states::AppState;
use crate::core::utils::{NoRotationChildCmp, NoRotationParentCmp};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use rand::{rng, Rng};
use std::f32::consts::PI;

#[derive(Component)]
pub struct TileCmp;

#[derive(Component)]
pub struct LeafCmp;

#[derive(Event)]
pub struct SpawnTileEv {
    pub tile: Tile,
    pub pos: Option<Vec2>,
}

fn _spawn_tile(
    commands: &mut Commands,
    tile: &Tile,
    pos: Vec2,
    color: Color,
    background: &Background,
    assets: &Local<WorldAssets>,
) {
    let texture = assets.texture(format!("tiles-{}", background.to_lowername()).as_str());

    let id = commands
        .spawn((
            Sprite {
                image: texture.image,
                custom_size: Some(Vec2::splat(Tile::SIZE)),
                color,
                texture_atlas: Some(TextureAtlas {
                    layout: texture.layout,
                    index: tile.texture_index,
                }),
                ..default()
            },
            Transform {
                translation: pos.extend(if tile.is_soil() {
                    TILE_Z_SCORE + 0.2 // On top of the base
                } else {
                    TILE_Z_SCORE
                }),
                rotation: Quat::from_rotation_z((-tile.rotation as f32).to_radians()),
                ..default()
            },
            NoRotationParentCmp,
            TileCmp,
            MapCmp,
        ))
        .observe(select_loc_on_click)
        .with_children(|parent| {
            if tile.has_stone {
                parent.spawn((
                    Sprite {
                        image: assets.image(&format!("stone{}", rng().random_range(1..=18))),
                        ..default()
                    },
                    Transform {
                        translation: Vec3::new(0., 0., 0.1),
                        rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                        scale: Vec3::splat(rng().random_range(0.2..0.25)),
                        ..default()
                    },
                ));
            }

            if let Some(leaf) = &tile.leaf {
                parent
                    .spawn((
                        Sprite {
                            image: assets.image(&leaf.image),
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(0., 0., 0.2),
                            scale: Vec3::splat((leaf.quantity / 1e3).max(0.1).min(0.3)),
                            ..default()
                        },
                        TeamCmp(LEAF_TEAM),
                        LeafCmp,
                        NoRotationChildCmp,
                    ))
                    .observe(select_leaf_on_click);
            }
        })
        .id();

    if tile.x != NON_MAP_ID {
        commands.entity(id).insert(tile.clone());
    }
}

pub fn spawn_tile_event(
    mut commands: Commands,
    tile_q: Query<(Entity, &Tile)>,
    app_state: Res<State<AppState>>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    mut map: ResMut<Map>,
    mut spawn_tile_ev: EventReader<SpawnTileEv>,
    mut server_send_message: EventWriter<ServerSendMessage>,
    mut client_send_message: EventWriter<ClientSendMessage>,
    assets: Local<WorldAssets>,
) {
    for SpawnTileEv { tile, pos } in spawn_tile_ev.read() {
        let background = if *app_state.get() == AppState::Game {
            game_settings.background
        } else {
            Background::Soil
        };

        let color = if *app_state.get() != AppState::Game
            || (game_settings.fog_of_war != FogOfWar::None
                && !players.main().visible_tiles.contains(&(tile.x, tile.y)))
        {
            NO_VISION_COLOR
        } else {
            Color::WHITE
        };

        // Check if there already exists a tile at the same position
        if let Some((tile_e, tile_c)) = tile_q.iter().find(|(_, t)| t.x == tile.x && t.y == tile.y)
        {
            // If the tile is not soil and the texture, rotation or leaf is different, replace it
            if !tile.is_soil()
                && (tile_c.texture_index != tile.texture_index
                    || tile_c.rotation != tile.rotation
                    || (tile_c.leaf.is_some() && tile.leaf.is_none())
                    || (tile_c.leaf.is_none() && tile.leaf.is_some()))
            {
                commands.entity(tile_e).despawn_recursive();

                // Delete the cache entries from the map that contain this tile
                map.cache.invalidate(&(tile_c.x, tile_c.y));

                _spawn_tile(
                    &mut commands,
                    &tile,
                    Map::get_coord_from_xy(tile_c.x, tile_c.y),
                    color,
                    &background,
                    &assets,
                );

                // Only send tiles that are visible by the player or a npc
                for player in players.0.iter().filter(|p| p.id == players.main_id() || p.is_npc()) {
                    if player.visible_tiles.contains(&(tile_c.x, tile_c.y)) {
                        server_send_message.send(ServerSendMessage {
                            message: ServerMessage::TileUpdate(tile.clone()),
                            client: None,
                        });

                        client_send_message.send(ClientSendMessage {
                            message: ClientMessage::TileUpdate(tile.clone()),
                        });

                        break;
                    }
                }
            }
        } else if let Some(pos) = pos {
            _spawn_tile(&mut commands, &tile, *pos, color, &background, &assets);
        }
    }
}
