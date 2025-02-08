use crate::core::menu::main::NPlayersCmp;
use crate::core::states::GameState;
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::{ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent};
use std::net::UdpSocket;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

const PROTOCOL_ID: u64 = 7;

#[derive(Resource)]
pub struct Player(pub u8);

#[derive(Event)]
pub struct NPlayersEv(u8);

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    StartGame(u8),
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
    server: Res<RenetServer>,
    mut server_ev: EventReader<ServerEvent>,
    mut n_players_ev: EventWriter<NPlayersEv>,
) {
    for event in server_ev.read() {
        n_players_ev.send(NPlayersEv(server.clients_id().len() as u8 + 1));
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {client_id} connected.");
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {client_id} disconnected: {reason}");
            }
        }
    }
}

pub fn server_events(
    mut n_players_ev: EventReader<NPlayersEv>,
    mut n_players_q: Query<&mut Text, With<NPlayersCmp>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for ev in n_players_ev.read() {
        if let Ok(mut text) = n_players_q.get_single_mut() {
            if ev.0 > 1 {
                text.0 = format!("There are {} players in the lobby...", ev.0);
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
    mut client: ResMut<RenetClient>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize(&message).unwrap() {
            ServerMessage::StartGame(i) => {
                commands.insert_resource(Player(i));
                next_game_state.set(GameState::Game);
            }
        }
    }
}
