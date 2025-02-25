mod ants;
mod assets;
mod audio;
mod camera;
mod constants;
mod map;
mod menu;
mod network;
mod pause;
mod player;
mod resources;
mod states;
mod utils;

use crate::core::ants::systems::*;
use crate::core::audio::*;
use crate::core::camera::{move_camera, move_camera_keyboard, setup_camera};
use crate::core::map::map::Map;
use crate::core::map::systems::{draw_map, MapCmp};
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::setup_menu;
use crate::core::network::{client_receive_message, server_update};
use crate::core::pause::{pause_game, spawn_pause_banner, toggle_pause_keyboard, unpause_game};
use crate::core::player::Player;
use crate::core::resources::GameSettings;
use crate::core::states::{GameState, MusicState, PauseState};
use crate::core::utils::despawn;
use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};

pub struct GamePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InGameSet;

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
            .init_resource::<Map>()
            //Sets
            .configure_sets(Update, InGameSet.run_if(in_state(GameState::Game)))
            // Camera
            .add_systems(Startup, (setup_camera, draw_map).chain())
            .add_systems(
                Update,
                (move_camera, move_camera_keyboard).in_set(InGameSet),
            )
            // Audio
            .add_systems(Startup, setup_music_btn)
            .add_systems(OnEnter(MusicState::Playing), play_music)
            .add_systems(OnEnter(MusicState::Stopped), stop_music)
            .add_systems(Update, (toggle_music, toggle_music_keyboard))
            //Networking
            .add_systems(
                PreUpdate,
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
            (despawn::<MapCmp>, draw_map).chain(),
        )
        // Pause
        .add_systems(Startup, spawn_pause_banner)
        .add_systems(OnEnter(PauseState::Paused), pause_game)
        .add_systems(OnEnter(PauseState::Running), unpause_game)
        .add_systems(Update, toggle_pause_keyboard.in_set(InGameSet))
        // Ants
        .add_systems(
            Update,
            (
                check_keys,
                hatch_eggs,
                animate_ants,
                resolve_action_ants,
                update_ant_health_bars,
                resolve_digging,
            )
                .run_if(in_state(PauseState::Running)),
        );
    }
}
