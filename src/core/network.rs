use crate::core::map::map::Map;
use crate::core::menu::buttons::LobbyTextCmp;
use crate::core::player::Player;
use crate::core::resources::GameSettings;
use crate::core::states::{GameState, PauseState};
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::{ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent};
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::time::SystemTime;

const PROTOCOL_ID: u64 = 7;

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    NPlayers(usize),
    StartGame {
        id: usize,
        settings: GameSettings,
        map: Map,
    },
    PauseGame,
    ResumeGame,
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
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for _ in server_ev.read() {
        let n_players = server.clients_id().len() + 1;

        // Update the number of players in the lobby
        let message = bincode::serialize(&ServerMessage::NPlayers(n_players)).unwrap();
        server.broadcast_message(DefaultChannel::ReliableOrdered, message);

        if let Ok(mut text) = n_players_q.get_single_mut() {
            if n_players > 1 {
                text.0 = format!("There are {n_players} players in the lobby...");
                next_game_state.set(GameState::ConnectedLobby);
            } else {
                text.0 = "Waiting for other players to join...".to_string();
                next_game_state.set(GameState::Lobby);
            }
        }
    }
}

pub fn client_receive_message(
    mut commands: Commands,
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut client: ResMut<RenetClient>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::NPlayers(i) => {
                if let Ok(mut text) = n_players_q.get_single_mut() {
                    text.0 = format!("There are {i} players in the lobby.\nWaiting for the host to start the game...");
                }
            }
            ServerMessage::StartGame { id, settings, map } => {
                commands.insert_resource(settings);
                commands.insert_resource(Player::new(id));
                commands.insert_resource(map);
                next_game_state.set(GameState::Game);
            }
            ServerMessage::PauseGame => {
                next_pause_state.set(PauseState::Paused);
            }
            ServerMessage::ResumeGame => {
                next_pause_state.set(PauseState::Running);
            }
        }
    }
}
