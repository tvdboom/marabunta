use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::{ConnectionConfig, RenetClient, RenetServer, ServerEvent};
use std::net::UdpSocket;
use std::time::SystemTime;
use crate::core::menu::main::NPlayersCmp;

const PROTOCOL_ID: u64 = 7;

#[derive(Event)]
pub struct NPlayersEv;

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
    let socket = UdpSocket::bind(public_addr).unwrap();
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
    mut server_ev: EventReader<ServerEvent>,
    mut n_players_ev: EventWriter<NPlayersEv>,
) {
    for event in server_ev.read() {
        n_players_ev.send(NPlayersEv);
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

pub fn network_events(
    server: Res<RenetServer>,
    mut n_players_ev: EventReader<NPlayersEv>,
    mut n_players_q: Query<&mut Text, With<NPlayersCmp>>,
) {
    for _ in n_players_ev.read() {
        if let Ok(mut text) = n_players_q.get_single_mut() {
            text.0 = format!("There are {} players in the lobby...", server.clients_id().len() + 1)
        }
    }
}
