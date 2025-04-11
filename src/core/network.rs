use crate::core::ants::events::{SpawnAntEv, SpawnEggEv};
use crate::core::audio::PlayAudioEv;
use crate::core::game_settings::{GameMode, GameSettings};
use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use crate::core::menu::buttons::LobbyTextCmp;
use crate::core::menu::settings::{Background, FogOfWar};
use crate::core::multiplayer::UpdatePopulationEv;
use crate::core::persistence::{GameLoaded, Population};
use crate::core::player::{Player, Players};
use crate::core::states::{AppState, GameState};
use crate::core::traits::AfterTraitCount;
use crate::utils::get_local_ip;
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::*;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::time::SystemTime;

const PROTOCOL_ID: u64 = 7;

#[derive(Event)]
pub struct ServerSendMessage {
    pub message: ServerMessage,
    pub client: Option<ClientId>,
}

#[derive(Event)]
pub struct ClientSendMessage {
    pub message: ClientMessage,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    LoadGame {
        background: Background,
        fog_of_war: FogOfWar,
        player: Player,
        map: Map,
        population: Population,
    },
    NPlayers(usize),
    StartGame {
        id: ClientId,
        background: Background,
        fog_of_war: FogOfWar,
        map: Map,
    },
    State(GameState),
    Status {
        speed: f32,
        population: Population,
    },
    TileUpdate(Tile),
}

impl ServerMessage {
    pub fn channel(&self) -> DefaultChannel {
        match self {
            ServerMessage::LoadGame { .. }
            | ServerMessage::NPlayers(_)
            | ServerMessage::StartGame { .. }
            | ServerMessage::State(_) => DefaultChannel::ReliableOrdered,
            ServerMessage::Status { .. } => DefaultChannel::Unreliable,
            ServerMessage::TileUpdate(_) => DefaultChannel::ReliableUnordered,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    State(GameState),
    Status {
        player: Player,
        population: Population,
    },
    TileUpdate(Tile),
    TileExplored {
        tile: (u32, u32),
        client: ClientId,
    },
}

impl ClientMessage {
    pub fn channel(&self) -> DefaultChannel {
        match self {
            ClientMessage::State(_) => DefaultChannel::ReliableOrdered,
            ClientMessage::Status { .. } => DefaultChannel::Unreliable,
            ClientMessage::TileUpdate(_) | ClientMessage::TileExplored { .. } => {
                DefaultChannel::ReliableUnordered
            }
        }
    }
}

pub fn new_renet_client(ip: &String) -> (RenetClient, NetcodeClientTransport) {
    let server_addr = format!("{ip}:5000").parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
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
    let public_addr = "0.0.0.0:5000".parse().unwrap();
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
                if n_players == 1 {
                    text.0 = format!("Waiting for other players to join {}...", get_local_ip());
                    next_app_state.set(AppState::Lobby);
                } else {
                    text.0 = format!("There are {n_players} players in the lobby.\nWaiting for other players to join {}...", get_local_ip());
                    next_app_state.set(AppState::ConnectedLobby);
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

pub fn server_send_message(
    mut server_send_message: EventReader<ServerSendMessage>,
    mut server: ResMut<RenetServer>,
) {
    for ev in server_send_message.read() {
        let message = bincode::serialize(&ev.message).unwrap();
        if let Some(client_id) = ev.client {
            server.send_message(client_id, ev.message.channel(), message);
        } else {
            server.broadcast_message(ev.message.channel(), message);
        }
    }
}

pub fn server_receive_message(
    mut server: ResMut<RenetServer>,
    mut players: ResMut<Players>,
    mut map: ResMut<Map>,
    mut trait_count: ResMut<AfterTraitCount>,
    mut update_population_ev: EventWriter<UpdatePopulationEv>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for id in server.clients_id() {
        while let Some(message) = server.receive_message(id, DefaultChannel::ReliableOrdered) {
            match bincode::deserialize(&message).unwrap() {
                ClientMessage::State(state) => match state {
                    GameState::InGameMenu | GameState::Paused
                        if *game_state.get() == GameState::Running =>
                    {
                        next_game_state.set(GameState::Paused);
                    }
                    GameState::Running => {
                        next_game_state.set(GameState::Running);
                    }
                    GameState::AfterTraitSelection => {
                        trait_count.0 += 1;
                    }
                    _ => (),
                },
                _ => unreachable!(),
            }
        }

        while let Some(message) = server.receive_message(id, DefaultChannel::ReliableUnordered) {
            match bincode::deserialize(&message).unwrap() {
                ClientMessage::TileUpdate(tile) => {
                    map.replace_tile(&tile);
                }
                ClientMessage::TileExplored { tile, client } => {
                    map.get_tile_mut(tile.0, tile.1)
                        .unwrap()
                        .explored
                        .insert(client);
                }
                _ => unreachable!(),
            }
        }

        while let Some(message) = server.receive_message(id, DefaultChannel::Unreliable) {
            match bincode::deserialize(&message).unwrap() {
                ClientMessage::Status { player, population } => {
                    if let Some(p) = players.0.iter_mut().find(|e| e.id == player.id) {
                        *p = player;
                    }
                    update_population_ev.send(UpdatePopulationEv { population, id });
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn client_send_message(
    mut client_send_message: EventReader<ClientSendMessage>,
    mut client: ResMut<RenetClient>,
) {
    for ev in client_send_message.read() {
        let message = bincode::serialize(&ev.message).unwrap();
        client.send_message(ev.message.channel(), message);
    }
}

pub fn client_receive_message(
    mut commands: Commands,
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut client: ResMut<RenetClient>,
    mut game_settings: ResMut<GameSettings>,
    mut map: ResMut<Map>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut spawn_egg_ev: EventWriter<SpawnEggEv>,
    game_state: Res<State<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut update_population_ev: EventWriter<UpdatePopulationEv>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::LoadGame {
                background,
                fog_of_war,
                player,
                map,
                population,
            } => {
                *game_settings = GameSettings {
                    game_mode: GameMode::Multiplayer,
                    background,
                    fog_of_war,
                    ..game_settings.clone()
                };

                commands.insert_resource(Players(Vec::from([player, Player::default()])));

                for (_, (transform, ant)) in population.ants {
                    spawn_ant_ev.send(SpawnAntEv {
                        ant,
                        transform,
                        entity: None,
                    });
                }
                for (_, (transform, egg)) in population.eggs {
                    spawn_egg_ev.send(SpawnEggEv {
                        ant: egg.ant,
                        transform,
                        entity: None,
                    });
                }

                commands.insert_resource(map);

                // Indicate the draw_map system to not load the starting queen
                commands.insert_resource(GameLoaded);

                next_app_state.set(AppState::Game);
            }
            ServerMessage::NPlayers(i) => {
                if let Ok(mut text) = n_players_q.get_single_mut() {
                    text.0 = format!("There are {i} players in the lobby.\nWaiting for the host to start the game...");
                }
            }
            ServerMessage::StartGame {
                id,
                background,
                fog_of_war,
                map,
            } => {
                *game_settings = GameSettings {
                    game_mode: GameMode::Multiplayer,
                    background,
                    fog_of_war,
                    ..game_settings.clone()
                };

                commands.insert_resource(Players(Vec::from([
                    Player::new(id, game_settings.color),
                    Player::default(),
                ])));

                commands.insert_resource(map);
                next_app_state.set(AppState::Game);
            }
            ServerMessage::State(state) => match state {
                GameState::InGameMenu | GameState::Paused
                    if *game_state.get() == GameState::Running =>
                {
                    next_game_state.set(GameState::Paused)
                }
                s @ GameState::Running | s @ GameState::TraitSelection => next_game_state.set(s),
                _ => (),
            },
            _ => unreachable!(),
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::ReliableUnordered) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::TileUpdate(tile) => {
                map.replace_tile(&tile);
            }
            _ => unreachable!(),
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::Status { speed, population } => {
                game_settings.speed = speed;
                update_population_ev.send(UpdatePopulationEv { population, id: 0 });
            }
            _ => unreachable!(),
        }
    }
}
