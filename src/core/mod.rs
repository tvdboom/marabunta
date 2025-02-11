mod assets;
mod audio;
mod camera;
mod input;
mod map;
mod menu;
mod network;
mod player;
mod resources;
mod states;

use crate::core::audio::{play_music, setup_music_btn, stop_music, toggle_music, ToggleMusicEv};
use crate::core::camera::{move_camera, setup_camera};
use crate::core::input::keys_listener;
use crate::core::map::systems::{draw_start_map, MapCmp};
use crate::core::menu::main::{setup_menu, update_lobby, MenuCmp};
use crate::core::menu::utils::despawn;
use crate::core::network::{client_receive_message, server_events, server_update, NPlayersEv};
use crate::core::states::{GameState, MusicState, PauseState};
use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .init_state::<GameState>()
            .init_state::<PauseState>()
            .init_state::<MusicState>()
            //Events
            .add_event::<ToggleMusicEv>()
            .add_event::<NPlayersEv>()
            // Camera
            .add_systems(Startup, (setup_camera, draw_start_map).chain())
            .add_systems(Update, move_camera)
            // Keyboard
            .add_systems(Update, keys_listener)
            // Audio
            .add_systems(Startup, setup_music_btn)
            .add_systems(OnEnter(MusicState::Playing), play_music)
            .add_systems(OnEnter(MusicState::Stopped), stop_music)
            .add_systems(Update, toggle_music)
            //Networking
            .add_systems(
                Update,
                (
                    (server_update, server_events).run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                ),
            );

        // Menu
        for state in [GameState::Menu, GameState::Lobby, GameState::ConnectedLobby] {
            app.add_systems(OnEnter(state), setup_menu)
                .add_systems(Update, update_lobby)
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }

        // Game
        app.add_systems(
            OnEnter(GameState::Game),
            (despawn::<MapCmp>, draw_start_map).chain(),
        );
    }
}
