use crate::core::ants::components::{AntCmp, Egg, Owned};
use crate::core::ants::events::{DespawnAntEv, SpawnAntEv, SpawnEggEv};
use crate::core::audio::PlayAudioEv;
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use crate::core::menu::buttons::LobbyTextCmp;
use crate::core::menu::settings::FogOfWar;
use crate::core::persistence::Population;
use crate::core::player::{Player, Players};
use crate::core::states::{AppState, GameState};
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::*;
use bimap::BiMap;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::time::SystemTime;

const PROTOCOL_ID: u64 = 7;

#[derive(Resource, Default)]
pub struct EntityMap(pub BiMap<Entity, Entity>);

#[derive(Event)]
pub struct UpdatePopulationEv(pub Population);

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    NPlayers(usize),
    StartGame {
        id: ClientId,
        fog_of_war: FogOfWar,
        map: Map,
    },
    Status {
        speed: f32,
        map: Map,
        population: Population,
        game_state: GameState,
    },
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Status { map: Map, population: Population },
}

pub fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());

    (client, transport)
}

pub fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).expect("Socket already in use.");
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 4,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(ConnectionConfig::default());

    (server, transport)
}

pub fn server_update(
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut server: ResMut<RenetServer>,
    mut server_ev: EventReader<ServerEvent>,
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for ev in server_ev.read() {
        if *app_state != AppState::Game {
            let n_players = server.clients_id().len() + 1;

            // Update the number of players in the lobby
            let message = bincode::serialize(&ServerMessage::NPlayers(n_players)).unwrap();
            server.broadcast_message(DefaultChannel::ReliableOrdered, message);

            if let Ok(mut text) = n_players_q.get_single_mut() {
                if n_players > 1 {
                    text.0 = format!("There are {n_players} players in the lobby...");
                    next_app_state.set(AppState::ConnectedLobby);
                } else {
                    text.0 = "Waiting for other players to join...".to_string();
                    next_app_state.set(AppState::Lobby);
                }
            }
        } else {
            match ev {
                ServerEvent::ClientConnected { client_id } => {
                    println!("Client {client_id} connected");
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    println!("Client {client_id} disconnected: {reason}");
                    play_audio_ev.send(PlayAudioEv {
                        name: "error",
                        volume: 0.5,
                    });
                    next_game_state.set(GameState::InGameMenu);
                }
            }
        }
    }
}

pub fn server_send_status(
    mut server: ResMut<RenetServer>,
    ant_q: Query<(Entity, &Transform, &AntCmp)>,
    egg_q: Query<(Entity, &Transform, &Egg)>,
    game_settings: Res<GameSettings>,
    map: Res<Map>,
    game_state: Res<State<GameState>>,
) {
    let status = bincode::serialize(&ServerMessage::Status {
        speed: game_settings.speed,
        map: map.clone(),
        population: Population {
            ants: ant_q
                .iter()
                .map(|(e, t, a)| (e, (t.clone(), a.clone())))
                .collect(),
            eggs: egg_q
                .iter()
                .map(|(e, t, a)| (e, (t.clone(), a.clone())))
                .collect(),
        },
        game_state: *game_state.get(),
    });

    server.broadcast_message(DefaultChannel::Unreliable, status.unwrap());
}

pub fn server_receive_status(
    mut server: ResMut<RenetServer>,
    mut map: ResMut<Map>,
    mut update_population_ev: EventWriter<UpdatePopulationEv>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            match bincode::deserialize(&message).unwrap() {
                ClientMessage::Status {
                    map: new_map,
                    population,
                } => {
                    map.update(new_map);
                    update_population_ev.send(UpdatePopulationEv(population));
                }
            }
        }
    }
}

pub fn client_send_status(
    mut client: ResMut<RenetClient>,
    ant_q: Query<(Entity, &Transform, &AntCmp)>,
    egg_q: Query<(Entity, &Transform, &Egg)>,
    players: Res<Players>,
    map: Res<Map>,
) {
    let status = bincode::serialize(&ClientMessage::Status {
        map: map.clone(),
        population: Population {
            ants: ant_q
                .iter()
                .filter_map(|(e, t, a)| {
                    (a.team == players.main_id()).then_some((e, (t.clone(), a.clone())))
                })
                .collect(),
            eggs: egg_q
                .iter()
                .filter_map(|(e, t, egg)| {
                    (egg.team == players.main_id()).then_some((e, (t.clone(), egg.clone())))
                })
                .collect(),
        },
    });
    client.send_message(DefaultChannel::Unreliable, status.unwrap());
}

pub fn client_receive_message(
    mut commands: Commands,
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut client: ResMut<RenetClient>,
    mut game_settings: ResMut<GameSettings>,
    mut map: ResMut<Map>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut update_population_ev: EventWriter<UpdatePopulationEv>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::NPlayers(i) => {
                if let Ok(mut text) = n_players_q.get_single_mut() {
                    text.0 = format!("There are {i} players in the lobby.\nWaiting for the host to start the game...");
                }
            }
            ServerMessage::StartGame {
                id,
                fog_of_war,
                map,
            } => {
                game_settings.fog_of_war = fog_of_war;

                commands.insert_resource(Players(Vec::from([
                    Player::new(id, game_settings.color),
                    Player::default(),
                ])));

                commands.insert_resource(map);
                next_app_state.set(AppState::Game);
            }
            _ => (),
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::Status {
                speed,
                map: map_status,
                population,
                game_state,
            } => {
                game_settings.speed = speed;
                map.update(map_status);

                next_game_state.set(if game_state == GameState::InGameMenu {
                    GameState::Paused
                } else {
                    game_state
                });

                update_population_ev.send(UpdatePopulationEv(population));
            }
            _ => (),
        }
    }
}

pub fn update_population_event(
    mut update_population_ev: EventReader<UpdatePopulationEv>,
    mut ant_q: Query<(Entity, &mut Transform, &mut AntCmp), Without<Owned>>,
    mut egg_q: Query<(Entity, &mut Transform, &mut Egg), (Without<Owned>, Without<AntCmp>)>,
    players: Res<Players>,
    entity_map: Res<EntityMap>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut spawn_egg_ev: EventWriter<SpawnEggEv>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
) {
    for UpdatePopulationEv(population) in update_population_ev.read() {
        // Despawn all that are not in the new population
        for (ant_e, _, _) in &ant_q {
            if !population
                .ants
                .contains_key(entity_map.0.get_by_right(&ant_e).unwrap())
            {
                despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
            }
        }

        for (egg_e, _, _) in &egg_q {
            if !population
                .eggs
                .contains_key(entity_map.0.get_by_right(&egg_e).unwrap())
            {
                despawn_ant_ev.send(DespawnAntEv { entity: egg_e });
            }
        }

        // Update the current population
        for (entity, (t, a)) in population
            .ants
            .iter()
            .filter(|(_, (_, a))| a.team != players.main_id())
        {
            if let Some(ant_e) = entity_map.0.get_by_left(entity) {
                if let Ok((_, mut ant_t, mut ant)) = ant_q.get_mut(*ant_e) {
                    *ant_t = *t;
                    *ant = a.clone();
                }
            } else {
                spawn_ant_ev.send(SpawnAntEv {
                    ant: a.clone(),
                    transform: *t,
                    entity: Some(*entity),
                });
            }
        }

        for (entity, (t, e)) in population
            .eggs
            .iter()
            .filter(|(_, (_, e))| e.team != players.main_id())
        {
            if let Some(egg_e) = entity_map.0.get_by_left(entity) {
                if let Ok((_, mut egg_t, mut egg)) = egg_q.get_mut(*egg_e) {
                    *egg_t = *t;
                    *egg = e.clone();
                }
            } else {
                spawn_egg_ev.send(SpawnEggEv {
                    ant: e.ant.clone(),
                    transform: *t,
                    entity: Some(*entity),
                });
            }
        }
    }
}
