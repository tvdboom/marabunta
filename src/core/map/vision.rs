use crate::core::ants::components::{AntCmp, Egg};
use crate::core::game_settings::GameSettings;
use crate::core::map::events::{LeafCmp, SpawnTileEv};
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::map::utils::reveal_tiles;
use crate::core::menu::settings::FogOfWar;
use crate::core::network::{ClientMessage, ClientSendMessage};
use crate::core::player::Players;
use bevy::color::Color;
use bevy::hierarchy::Children;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;

pub fn update_vision(
    mut ant_q: Query<(Entity, &mut Transform, &mut Visibility, &AntCmp)>,
    mut egg_q: Query<(Entity, &Transform, &mut Visibility, &Egg), Without<AntCmp>>,
    mut tile_q: Query<(Entity, &mut Sprite, &Tile)>,
    mut leaf_q: Query<&mut Sprite, (With<LeafCmp>, Without<Tile>)>,
    children_q: Query<&Children>,
    mut spawn_tile_ev: EventWriter<SpawnTileEv>,
    game_settings: Res<GameSettings>,
    mut players: ResMut<Players>,
    mut map: ResMut<Map>,
    mut client_send_message: EventWriter<ClientSendMessage>,
) {
    let id = players.main_id();
    for player in players.0.iter_mut().filter(|p| p.id == id || p.is_npc()) {
        player.visible_tiles = HashSet::new();

        // Calculate all tiles currently visible by the player
        ant_q
            .iter()
            .filter(|(_, _, _, a)| a.team == player.id && a.health > 0.)
            .for_each(|(_, ant_t, _, _)| {
                let current_tile = map.get_tile_from_coord(&ant_t.translation).unwrap();
                player
                    .visible_tiles
                    .extend(reveal_tiles(current_tile, &map, None, 0))
            });

        // Add stone tiles with 2 or more revealed neighbors to the list
        map.tiles.iter().filter(|t| t.has_stone).for_each(|t| {
            let visible_neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .filter(|(dx, dy)| {
                    let nx = t.x as i32 + dx;
                    let ny = t.y as i32 + dy;
                    nx >= 0 && ny >= 0 && player.visible_tiles.contains(&(nx as u32, ny as u32))
                })
                .count();

            if visible_neighbors >= 2 {
                player.visible_tiles.insert((t.x, t.y));
            }
        });

        // Add the exploration marker
        map.tiles
            .iter_mut()
            .filter(|t| player.visible_tiles.contains(&(t.x, t.y)))
            .for_each(|t| {
                t.explored.insert(player.id);

                // Only the server must know the explored tiles since it manages monsters
                client_send_message.send(ClientSendMessage {
                    message: ClientMessage::TileExplored {
                        tile: (t.x, t.y),
                        client: player.id,
                    },
                });
            });

        if player.is_human() {
            if game_settings.fog_of_war != FogOfWar::Full {
                // Spawn all tiles to keep the map up to date
                map.tiles.iter().for_each(|tile| {
                    spawn_tile_ev.send(SpawnTileEv {
                        tile: tile.clone(),
                        pos: None,
                    });
                });
            } else {
                // Spawn only visible tiles
                player.visible_tiles.iter().for_each(|(x, y)| {
                    let tile = map.get_tile_mut(*x, *y).unwrap();

                    tile.explored.insert(player.id);
                    spawn_tile_ev.send(SpawnTileEv {
                        tile: tile.clone(),
                        pos: None,
                    });
                });
            }

            // Adjust the fog of war on the map
            if game_settings.fog_of_war != FogOfWar::None {
                tile_q.iter_mut().for_each(|(tile_e, mut sprite, tile)| {
                    let color = if player.visible_tiles.contains(&(tile.x, tile.y)) {
                        Color::WHITE
                    } else {
                        Color::srgba(1., 1., 1., 0.5)
                    };

                    sprite.color = color;

                    // Update child (leaf) sprite color
                    if let Ok(children) = children_q.get(tile_e) {
                        for &child in children.iter() {
                            if let Ok(mut leaf_s) = leaf_q.get_mut(child) {
                                leaf_s.color = color;
                            }
                        }
                    }
                });

                // Show/hide enemies on the map
                for (_, ant_t, mut ant_v, ant) in &mut ant_q {
                    if ant.team != player.id {
                        if map
                            .get_tile_from_coord(&ant_t.translation)
                            .map_or(false, |tile| {
                                player.visible_tiles.contains(&(tile.x, tile.y))
                            })
                        {
                            // The enemy is visible, show it
                            *ant_v = Visibility::Inherited;
                        } else if ant.health > 0. {
                            // The enemy is no longer visible, hide it (unless corpse)
                            *ant_v = Visibility::Hidden;
                        }
                    }
                }

                for (_, egg_t, mut egg_v, egg) in &mut egg_q {
                    if egg.team != player.id {
                        if map
                            .get_tile_from_coord(&egg_t.translation)
                            .map_or(false, |tile| {
                                player.visible_tiles.contains(&(tile.x, tile.y))
                            })
                        {
                            *egg_v = Visibility::Inherited;
                        } else {
                            *egg_v = Visibility::Hidden;
                        }
                    }
                }
            }
        }
    }
}
