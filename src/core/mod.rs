mod ants;
mod assets;
mod audio;
mod camera;
mod map;
mod menu;
mod network;
mod pause;
mod player;
mod resources;
mod states;
mod utils;
mod constants;

use crate::core::ants::systems::{animate_ants, check_keys, hatch_eggs, resolve_action_ants, tile_dig};
use crate::core::audio::{
    play_music, setup_music_btn, stop_music, toggle_music, toggle_music_keyboard, ToggleMusicEv,
};
use crate::core::camera::{move_camera, move_camera_keyboard, setup_camera};
use crate::core::map::systems::{draw_start_map, MapCmp};
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::{setup_menu, spawn_menu_ants};
use crate::core::network::{client_receive_message, server_update};
use crate::core::pause::{pause_game, spawn_pause_banner, toggle_pause_keyboard, unpause_game};
use crate::core::resources::GameSettings;
use crate::core::states::{GameState, MusicState, PauseState};
use crate::core::utils::despawn;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_renet::renet::{RenetClient, RenetServer};
use std::time::Duration;
use crate::core::player::Player;

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
            //Resources
            .init_resource::<GameSettings>()
            .init_resource::<Player>()
            // Camera
            .add_systems(Startup, (setup_camera, draw_start_map).chain())
            .add_systems(Update, (move_camera, move_camera_keyboard))
            // Audio
            .add_systems(Startup, setup_music_btn)
            .add_systems(OnEnter(MusicState::Playing), play_music)
            .add_systems(OnEnter(MusicState::Stopped), stop_music)
            .add_systems(Update, (toggle_music, toggle_music_keyboard))
            //Networking
            .add_systems(
                Update,
                (
                    server_update.run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                ),
            );

        // Menu
        for state in [
            GameState::MainMenu,
            GameState::MultiPlayerMenu,
            GameState::Lobby,
            GameState::ConnectedLobby,
        ] {
            app.add_systems(OnEnter(state), setup_menu)
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }

        // Map
        app.add_systems(
            OnEnter(GameState::Game),
            (despawn::<MapCmp>, draw_start_map).chain(),
        )
        // Pause
        .add_systems(Startup, spawn_pause_banner)
        .add_systems(OnEnter(PauseState::Paused), pause_game)
        .add_systems(OnEnter(PauseState::Running), unpause_game)
        .add_systems(
            Update,
            toggle_pause_keyboard.run_if(in_state(GameState::Game)),
        )
        // Ants
        .add_systems(
            Update,
            (
                check_keys,
                hatch_eggs,
                animate_ants,
                resolve_action_ants,
                tile_dig,
                spawn_menu_ants.run_if(on_timer(Duration::from_millis(200))),
            )
                .run_if(in_state(PauseState::Running)),
        );
    }
}
