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

use crate::core::ants::events::*;
use crate::core::ants::systems::*;
use crate::core::audio::*;
use crate::core::camera::*;
use crate::core::map::events::{spawn_tile, SpawnTileEv};
use crate::core::map::systems::*;
use crate::core::map::ui::systems::{animate_ui, draw_ui, update_ui};
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::{setup_in_game_menu, setup_menu};
use crate::core::network::*;
use crate::core::pause::*;
use crate::core::states::{AppState, AudioState, GameState};
use crate::core::systems::{check_keys, initialize_game};
use crate::core::utils::{despawn, update_transform_no_rotation};
use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};

pub struct GamePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InRunningGameSet;

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
            .add_event::<QueueAntEv>()
            .add_event::<SpawnEggEv>()
            .add_event::<SpawnAntEv>()
            .add_event::<DespawnAntEv>()
            .add_event::<AttackEv>()
            .add_event::<DamageAntEv>()
            // Sets
            .configure_sets(PreUpdate, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(Update, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(PostUpdate, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(
                PreUpdate,
                InRunningGameSet
                    .run_if(in_state(GameState::Running))
                    .in_set(InGameSet),
            )
            .configure_sets(
                Update,
                InRunningGameSet
                    .run_if(in_state(GameState::Running))
                    .in_set(InGameSet),
            )
            .configure_sets(
                PostUpdate,
                InRunningGameSet
                    .run_if(in_state(GameState::Running))
                    .in_set(InGameSet),
            )
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
        .add_systems(
            OnEnter(AppState::Game),
            (despawn::<MapCmp>, draw_map, draw_ui),
        )
        .add_systems(Update, (animate_ui, update_ui).in_set(InRunningGameSet))
        .add_systems(
            OnExit(AppState::Game),
            (despawn::<MapCmp>, reset_camera, initialize_game, draw_map).chain(),
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
                resolve_attack_action,
                resolve_die_action,
                resolve_brood_action,
                resolve_idle_action,
                resolve_targeted_walk_action,
                resolve_walk_action,
                resolve_digging,
                resolve_harvesting,
                update_ant_components,
                update_vision,
            )
                .in_set(InRunningGameSet),
        )
        .add_systems(
            PostUpdate,
            (
                spawn_tile,
                (
                    queue_ant_event,
                    spawn_egg_event,
                    spawn_ant_event,
                    despawn_ant_event,
                    attack_ants,
                    damage_event,
                )
                    .in_set(InRunningGameSet),
            ),
        );
    }
}
