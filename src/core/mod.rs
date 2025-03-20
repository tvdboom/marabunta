mod ants;
mod assets;
mod audio;
mod camera;
mod constants;
mod game_settings;
mod map;
mod menu;
mod network;
mod pause;
mod persistence;
mod player;
mod resources;
mod states;
mod systems;
mod traits;
mod utils;

use crate::core::ants::events::*;
use crate::core::ants::selection::remove_command_from_selection;
use crate::core::ants::systems::*;
use crate::core::audio::*;
use crate::core::camera::*;
use crate::core::map::events::{spawn_tile, SpawnTileEv};
use crate::core::map::systems::*;
use crate::core::map::ui::systems::{animate_ui, draw_ui, update_ui, UiCmp};
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::{setup_game_over, setup_in_game_menu, setup_menu};
use crate::core::network::*;
use crate::core::pause::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::persistence::{load_game, save_game};
use crate::core::persistence::{LoadGameEv, SaveGameEv};
use crate::core::states::{AppState, AudioState, GameState};
use crate::core::systems::*;
use crate::core::traits::{select_trait_event, TraitSelectedEv};
use crate::core::utils::{despawn, update_transform_no_rotation};
use ants::selection::{select_ants_from_rect, select_ants_to_res, SelectAntEv};
use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};
use map::ui::systems::setup_trait_selection;
use strum::IntoEnumIterator;

pub struct GamePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InRunningGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InRunningOrPausedGameSet;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .init_state::<AppState>()
            .init_state::<GameState>()
            .init_state::<AudioState>()
            // Events
            .add_event::<ToggleAudioEv>()
            .add_event::<PlayAudioEv>()
            .add_event::<LoadGameEv>()
            .add_event::<SaveGameEv>()
            .add_event::<SpawnTileEv>()
            .add_event::<QueueAntEv>()
            .add_event::<SpawnEggEv>()
            .add_event::<SpawnAntEv>()
            .add_event::<DespawnAntEv>()
            .add_event::<DamageAntEv>()
            .add_event::<SelectAntEv>()
            .add_event::<TraitSelectedEv>()
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
            .configure_sets(
                PreUpdate,
                InRunningOrPausedGameSet
                    .run_if(in_state(GameState::Running).or(in_state(GameState::Paused)))
                    .in_set(InGameSet),
            )
            .configure_sets(
                Update,
                InRunningOrPausedGameSet
                    .run_if(in_state(GameState::Running).or(in_state(GameState::Paused)))
                    .in_set(InGameSet),
            )
            .configure_sets(
                PostUpdate,
                InRunningOrPausedGameSet
                    .run_if(in_state(GameState::Running).or(in_state(GameState::Paused)))
                    .in_set(InGameSet),
            )
            // Camera
            .add_systems(Startup, (setup_camera, initialize_game, draw_map).chain())
            .add_systems(
                Update,
                (move_camera, move_camera_keyboard).in_set(InRunningOrPausedGameSet),
            )
            // Audio
            .add_systems(Startup, setup_music_btn)
            .add_systems(OnEnter(AudioState::Sound), play_music)
            .add_systems(
                Update,
                (toggle_music_event, toggle_music_keyboard, play_audio_event),
            )
            //Networking
            .add_systems(
                First,
                (
                    (server_update, server_receive_status.in_set(InGameSet))
                        .run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                ),
            )
            .add_systems(
                Last,
                (
                    server_send_status.run_if(resource_exists::<RenetServer>),
                    client_send_status.run_if(resource_exists::<RenetClient>),
                )
                    .in_set(InGameSet),
            );

        // Menu
        for state in AppState::iter().filter(|s| *s != AppState::Game) {
            app.add_systems(OnEnter(state), setup_menu)
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }

        // Utilities
        app.add_systems(Update, check_keys.in_set(InGameSet))
            .add_systems(
                PostUpdate,
                (
                    on_resize_system,
                    update_transform_no_rotation.before(TransformSystem::TransformPropagate),
                ),
            )
            // Map
            .add_systems(
                OnEnter(AppState::Game),
                (despawn::<MapCmp>, draw_map, draw_ui),
            )
            .add_systems(Update, (animate_ui, update_ui).in_set(InGameSet))
            .add_systems(
                OnExit(AppState::Game),
                (despawn::<MapCmp>, reset_camera, initialize_game, draw_map).chain(),
            )
            // Selection
            .add_systems(
                PreUpdate,
                select_ants_from_rect.in_set(InRunningOrPausedGameSet),
            )
            .add_systems(Update, remove_command_from_selection)
            .add_systems(
                PostUpdate,
                select_ants_to_res.in_set(InRunningOrPausedGameSet),
            )
            // In-game states
            .add_systems(Startup, spawn_pause_banner)
            .add_systems(OnEnter(GameState::Paused), pause_game)
            .add_systems(OnExit(GameState::Paused), unpause_game)
            .add_systems(OnEnter(GameState::InGameMenu), setup_in_game_menu)
            .add_systems(OnExit(GameState::InGameMenu), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::TraitSelection), setup_trait_selection)
            .add_systems(
                OnExit(GameState::TraitSelection),
                (despawn::<MenuCmp>, despawn::<UiCmp>, draw_ui).chain(),
            )
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn::<MenuCmp>)
            .add_systems(Update, toggle_pause_keyboard.in_set(InGameSet))
            // Ants
            .add_systems(PreUpdate, resolve_pre_action.in_set(InRunningGameSet))
            .add_systems(
                Update,
                update_ant_components.in_set(InRunningOrPausedGameSet),
            )
            .add_systems(
                Update,
                (
                    check_trait_timer,
                    queue_ants_keyboard,
                    hatch_eggs,
                    animate_ants,
                    resolve_digging,
                    resolve_harvesting,
                    resolve_healing,
                    resolve_attack_action,
                    resolve_die_action,
                    resolve_brood_action,
                    resolve_idle_action,
                    resolve_targeted_walk_action,
                    resolve_walk_action,
                    update_vision,
                    spawn_enemies,
                )
                    .in_set(InRunningGameSet),
            )
            .add_systems(
                PostUpdate,
                (
                    spawn_tile,
                    select_trait_event,
                    (
                        queue_ant_event,
                        spawn_egg_event,
                        spawn_ant_event,
                        despawn_ant_event,
                        damage_event,
                    )
                        .in_set(InRunningGameSet),
                ),
            );

        // Persistence
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, (load_game, save_game));
    }
}
