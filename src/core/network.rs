use crate::core::ants::components::AntCmp;
use crate::core::map::map::Map;
use crate::core::menu::buttons::LobbyTextCmp;
use crate::core::player::Player;
use crate::core::resources::{GameSettings, Population, PopulationT};
use crate::core::states::{AppState, GameState};
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::{
    ClientId, ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent,
};
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::time::SystemTime;

const PROTOCOL_ID: u64 = 7;

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    NPlayers(usize),
    StartGame {
        id: ClientId,
        settings: GameSettings,
        map: Map,
    },
    Status {
        settings: GameSettings,
        pause: GameState,
        map: Map,
        population: PopulationT,
    },
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Status { map: Map, population: PopulationT },
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
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
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
                }
            }
        }
    }
}

pub fn server_send_status(
    mut server: ResMut<RenetServer>,
    ant_q: Query<(&AntCmp, &Transform)>,
    game_settings: Res<GameSettings>,
    map: Res<Map>,
    game_state: Res<State<GameState>>,
) {
    let status = bincode::serialize(&ServerMessage::Status {
        settings: game_settings.clone(),
        pause: *game_state.get(),
        map: map.clone(),
        population: ant_q
            .iter()
            .map(|(a, t)| (a.id, (t.clone(), a.clone())))
            .collect(),
    });

    server.broadcast_message(DefaultChannel::ReliableOrdered, status.unwrap());
}

pub fn server_receive_status(
    mut server: ResMut<RenetServer>,
    mut map: ResMut<Map>,
    mut population: ResMut<Population>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            match bincode::deserialize(&message).unwrap() {
                ClientMessage::Status {
                    map: new_map,
                    population: new_population,
                } => {
                    map.update(new_map);

                    // The server takes all population send by client and removes existing ones
                    population.0.retain(|_, (_, a)| a.owner != client_id);
                    population.0.extend(new_population);
                }
            }
        }
    }
}

pub fn client_send_status(
    mut client: ResMut<RenetClient>,
    ant_q: Query<(&Transform, &AntCmp)>,
    player: Res<Player>,
    map: Res<Map>,
) {
    let status = bincode::serialize(&ClientMessage::Status {
        map: map.clone(),
        population: ant_q
            .iter()
            .filter_map(|(t, a)| {
                if a.owner == player.id {
                    Some((a.id, (t.clone(), a.clone())))
                } else {
                    None
                }
            })
            .collect(),
    });
    client.send_message(DefaultChannel::ReliableOrdered, status.unwrap());
}

pub fn client_receive_message(
    mut commands: Commands,
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut client: ResMut<RenetClient>,
    player: Res<Player>,
    mut map: ResMut<Map>,
    mut population: ResMut<Population>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::NPlayers(i) => {
                if let Ok(mut text) = n_players_q.get_single_mut() {
                    text.0 = format!("There are {i} players in the lobby.\nWaiting for the host to start the game...");
                }
            }
            ServerMessage::StartGame { id, settings, map } => {
                commands.insert_resource(Player::new(id));
                commands.insert_resource(settings);
                commands.insert_resource(map);
                next_app_state.set(AppState::Game);
            }
            ServerMessage::Status {
                settings,
                mut pause,
                map: new_map,
                population: new_population,
            } => {
                commands.insert_resource(settings);
                map.update(new_map);

                if pause == GameState::InGameMenu {
                    pause = GameState::Paused;
                }
                next_game_state.set(pause);

                // The client takes all population not owned by self
                population.0 = new_population
                    .into_iter()
                    .filter(|(_, (_, a))| a.owner != player.id)
                    .collect();
            }
        }
    }
}
