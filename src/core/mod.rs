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
mod systems;
mod utils;

use crate::core::ants::events::{despawn_ants, spawn_ants, DespawnAntEv, SpawnAntEv};
use crate::core::ants::systems::*;
use crate::core::audio::*;
use crate::core::camera::*;
use crate::core::map::events::{spawn_tile, SpawnTileEv};
use crate::core::map::systems::*;
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::{setup_in_game_menu, setup_menu};
use crate::core::network::*;
use crate::core::pause::*;
use crate::core::states::{AppState, AudioState, GameState};
use crate::core::systems::{check_keys, initialize_game};
use crate::core::utils::{despawn, update_transform_no_rotation};
use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};
use map::ui::systems::{draw_ui, update_ui};

pub struct GamePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct RunningSet;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .init_state::<AppState>()
            .init_state::<GameState>()
            .init_state::<AudioState>()
            // Events
            .add_event::<ToggleMusicEv>()
            .add_event::<SpawnTileEv>()
            .add_event::<SpawnAntEv>()
            .add_event::<DespawnAntEv>()
            // Sets
            .configure_sets(PreUpdate, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(Update, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(PostUpdate, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(PreUpdate, RunningSet.run_if(in_state(GameState::Running)))
            .configure_sets(Update, RunningSet.run_if(in_state(GameState::Running)))
            .configure_sets(PostUpdate, RunningSet.run_if(in_state(GameState::Running)))
            // Camera
            .add_systems(Startup, (setup_camera, initialize_game, draw_map).chain())
            .add_systems(
                Update,
                (move_camera, move_camera_keyboard)
                    .run_if(not(in_state(GameState::InGameMenu)))
                    .in_set(InGameSet),
            )
            // Audio
            .add_systems(Startup, setup_music_btn)
            .add_systems(OnEnter(AudioState::Playing), play_music)
            .add_systems(OnEnter(AudioState::Stopped), stop_music)
            .add_systems(Update, (toggle_music, toggle_music_keyboard))
            //Networking
            .add_systems(
                PreUpdate,
                (
                    (server_update, server_receive_status.in_set(InGameSet))
                        .run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    server_send_status.run_if(resource_exists::<RenetServer>),
                    client_send_status.run_if(resource_exists::<RenetClient>),
                )
                    .in_set(InGameSet),
            );

        // Menu
        for state in [
            AppState::MainMenu,
            AppState::MultiPlayerMenu,
            AppState::Lobby,
            AppState::ConnectedLobby,
        ] {
            app.add_systems(OnEnter(state), setup_menu)
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }

        // Utilities
        app.add_systems(
            PostUpdate,
            update_transform_no_rotation.before(TransformSystem::TransformPropagate),
        )
        // Map
        .add_systems(OnEnter(AppState::Game), (despawn::<MapCmp>, draw_map, draw_ui))
        .add_systems(Update, update_ui)
        .add_systems(
            OnExit(AppState::Game),
            (despawn::<MapCmp>, initialize_game, draw_map, draw_ui).chain(),
        )
        // Pause
        .add_systems(Startup, spawn_pause_banner)
        .add_systems(OnEnter(GameState::Paused), pause_game)
        .add_systems(OnExit(GameState::Paused), unpause_game)
        .add_systems(OnEnter(GameState::InGameMenu), setup_in_game_menu)
        .add_systems(OnExit(GameState::InGameMenu), despawn::<MenuCmp>)
        .add_systems(Update, toggle_pause_keyboard.in_set(InGameSet))
        // Ants
        .add_systems(
            Update,
            (
                check_keys,
                hatch_eggs,
                animate_ants,
                resolve_action_ants,
                update_ant_components,
                update_vision,
                resolve_digging,
                resolve_harvesting,
            )
                .in_set(RunningSet),
        )
        .add_systems(
            PostUpdate,
            (spawn_tile, spawn_ants, despawn_ants).in_set(RunningSet),
        );
    }
}
