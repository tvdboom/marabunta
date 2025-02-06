use std::net::UdpSocket;
use std::time::SystemTime;
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::{ConnectionConfig, RenetClient, RenetServer, ServerEvent};

const PROTOCOL_ID: u64 = 7;

pub fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
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
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
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
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in server_events.read() {
        println!("{:?}", event);
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.
                // for &player_id in lobby.players.keys() {
                //     let message = bincode::serialize(&ServerMessages::PlayerConnected { id: player_id }).unwrap();
                //     server.send_message(*client_id, DefaultChannel::ReliableOrdered, message);
                // }
                //
                // lobby.players.insert(*client_id, player_entity);
                //
                // let message = bincode::serialize(&ServerMessages::PlayerConnected { id: *client_id }).unwrap();
                // server.broadcast_message(DefaultChannel::ReliableOrdered, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);
                // if let Some(player_entity) = lobby.players.remove(client_id) {
                //     commands.entity(player_entity).despawn();
                // }
                //
                // let message = bincode::serialize(&ServerMessages::PlayerDisconnected { id: *client_id }).unwrap();
                // server.broadcast_message(DefaultChannel::ReliableOrdered, message);
            }
        }
    }

    // for client_id in server.clients_id() {
    //     while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
    //         let player_input: PlayerInput = bincode::deserialize(&message).unwrap();
    //         if let Some(player_entity) = lobby.players.get(&client_id) {
    //             commands.entity(*player_entity).insert(player_input);
    //         }
    //     }
    // }
}