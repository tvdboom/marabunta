use crate::core::ants::components::{AntCmp, Egg};
use crate::core::ants::events::{SpawnAntEv, SpawnEggEv};
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use crate::core::network::{ServerMessage, ServerSendMessage};
use crate::core::player::Players;
use crate::core::states::{AppState, AudioState};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::RenetServer;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::{Read, Write};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Population {
    pub ants: HashMap<Entity, (Transform, AntCmp)>,
    pub eggs: HashMap<Entity, (Transform, Egg)>,
}

#[derive(Serialize, Deserialize)]
pub struct SaveAll {
    pub game_settings: GameSettings,
    pub players: Players,
    pub map: Map,
    pub population: Population,
}

#[derive(Event)]
pub struct LoadGameEv;

#[derive(Event)]
pub struct SaveGameEv;

#[derive(Resource)]
pub struct GameLoaded;

fn save_to_bin(file_path: &str, data: &SaveAll) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(&bincode::serialize(data).expect("Failed to serialize data."))?;
    Ok(())
}

fn load_from_bin(file_path: &str) -> io::Result<SaveAll> {
    let mut file = File::open(file_path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    let data: SaveAll = bincode::deserialize(&buffer).expect("Failed to deserialize data.");
    Ok(data)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game(
    mut commands: Commands,
    server: Option<Res<RenetServer>>,
    mut load_game_ev: EventReader<LoadGameEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut spawn_egg_ev: EventWriter<SpawnEggEv>,
    mut server_send_message: EventWriter<ServerSendMessage>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_audio_state: ResMut<NextState<AudioState>>,
) {
    for _ in load_game_ev.read() {
        if let Some(file_path) = FileDialog::new().pick_file() {
            let file_path_str = file_path.to_string_lossy().to_string();
            let mut data = load_from_bin(&file_path_str).expect("Failed to load the game.");

            let ids = data
                .players
                .0
                .iter()
                .filter_map(|p| p.is_human().then_some(p.id))
                .collect::<Vec<_>>();

            let n_humans = ids.len();
            if n_humans > 1 {
                if let Some(server) = server.as_ref() {
                    let n_clients = server.clients_id().len();
                    if n_clients != n_humans - 1 {
                        panic!("The loaded game contains {n_humans} players but the server has {} players.", n_clients + 1);
                    } else {
                        for (new_id, old_id) in server.clients_id().iter().zip(ids.iter().skip(1)) {
                            let player =
                                data.players.0.iter_mut().find(|p| p.id == *old_id).unwrap();

                            // Update everything to the new player id
                            player.id = *new_id;

                            data.map.tiles.iter_mut().for_each(|tile| {
                                if tile.base.is_some_and(|id| id == *old_id) {
                                    tile.base = Some(*new_id);
                                }

                                if tile.explored.remove(old_id) {
                                    tile.explored.insert(*new_id);
                                }
                            });

                            data.population.ants.iter_mut().for_each(|(_, (_, a))| {
                                if a.team == *old_id {
                                    a.team = *new_id;
                                }
                            });

                            data.population.eggs.iter_mut().for_each(|(_, (_, e))| {
                                if e.team == *old_id {
                                    e.team = *new_id;
                                    e.ant.team = *new_id;
                                }
                            });

                            server_send_message.send(ServerSendMessage {
                                message: ServerMessage::LoadGame {
                                    background: data.game_settings.background,
                                    fog_of_war: data.game_settings.fog_of_war,
                                    player: player.clone(),
                                    map: data.map.clone(),
                                    population: Population {
                                        ants: data
                                            .population
                                            .ants
                                            .clone()
                                            .into_iter()
                                            .filter(|(_, (_, a))| a.team == *new_id)
                                            .collect(),
                                        eggs: data
                                            .population
                                            .eggs
                                            .clone()
                                            .into_iter()
                                            .filter(|(_, (_, e))| e.team == *new_id)
                                            .collect(),
                                    },
                                },
                                client: Some(*new_id),
                            });
                        }
                    }
                } else {
                    panic!("The loaded game contains {n_humans} players but there is no server initiated.");
                }
            }

            next_audio_state.set(data.game_settings.audio);
            commands.insert_resource(data.game_settings);

            commands.insert_resource(data.players);
            commands.insert_resource(data.map);

            for (_, (transform, ant)) in data
                .population
                .ants
                .into_iter()
                .filter(|(_, (_, a))| a.team == 0)
            {
                spawn_ant_ev.send(SpawnAntEv {
                    ant,
                    transform,
                    entity: None,
                });
            }
            for (_, (transform, egg)) in data
                .population
                .eggs
                .into_iter()
                .filter(|(_, (_, e))| e.team == 0)
            {
                spawn_egg_ev.send(SpawnEggEv {
                    ant: egg.ant,
                    transform,
                    entity: None,
                });
            }

            // This resource indicates the draw_map system to not load the starting queen
            commands.insert_resource(GameLoaded);

            next_app_state.set(AppState::Game);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(
    mut save_game_ev: EventReader<SaveGameEv>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    map: Res<Map>,
    ant_q: Query<(Entity, &Transform, &AntCmp)>,
    egg_q: Query<(Entity, &Transform, &Egg)>,
) {
    for _ in save_game_ev.read() {
        if let Some(mut file_path) = FileDialog::new().save_file() {
            if !file_path.extension().map(|e| e == "bin").unwrap_or(false) {
                file_path.set_extension("bin");
            }

            let file_path_str = file_path.to_string_lossy().to_string();
            let data = SaveAll {
                game_settings: game_settings.clone(),
                players: players.clone(),
                map: map.clone(),
                population: Population {
                    ants: ant_q
                        .iter()
                        .map(|(e, t, a)| (e, (t.clone(), a.clone())))
                        .collect(),
                    eggs: egg_q
                        .iter()
                        .map(|(e, t, egg)| (e, (t.clone(), egg.clone())))
                        .collect(),
                },
            };

            save_to_bin(&file_path_str, &data).expect("Failed to save the game.");
        }
    }
}
