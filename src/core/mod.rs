mod ants;
mod assets;
mod audio;
mod camera;
mod constants;
mod game_settings;
mod map;
mod menu;
mod multiplayer;
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
use crate::core::ants::selection::*;
use crate::core::ants::systems::*;
use crate::core::audio::*;
use crate::core::camera::*;
use crate::core::constants::{ENEMY_TIMER, NETWORK_TIMER};
use crate::core::game_settings::GameSettings;
use crate::core::map::events::{spawn_tile_event, SpawnTileEv};
use crate::core::map::holes::{resolve_expeditions, spawn_enemies};
use crate::core::map::systems::*;
use crate::core::map::ui::systems::{animate_ui, draw_ui, setup_after_trait, update_ui, UiCmp};
use crate::core::map::vision::update_vision;
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::{setup_end_game, setup_in_game_menu, setup_menu, update_ip, Ip};
use crate::core::multiplayer::*;
use crate::core::network::*;
use crate::core::pause::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::persistence::{load_game, save_game};
use crate::core::persistence::{LoadGameEv, SaveGameEv};
use crate::core::states::{AppState, AudioState, GameState};
use crate::core::systems::*;
use crate::core::traits::{after_trait_check, select_trait_event, TraitSelectedEv};
use crate::core::utils::{despawn, update_transform_no_rotation};
use ants::selection::{select_ants_from_rect, select_ants_to_res, SelectAntEv};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_renet::renet::{RenetClient, RenetServer};
use map::ui::systems::setup_trait_selection;
use std::time::Duration;
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
            .add_event::<ChangeAudioEv>()
            .add_event::<PlayAudioEv>()
            .add_event::<LoadGameEv>()
            .add_event::<SaveGameEv>()
            .add_event::<SpawnTileEv>()
            .add_event::<QueueAntEv>()
            .add_event::<SpawnEggEv>()
            .add_event::<SpawnAntEv>()
            .add_event::<PinEv>()
            .add_event::<DespawnAntEv>()
            .add_event::<DamageAntEv>()
            .add_event::<SelectAntEv>()
            .add_event::<TraitSelectedEv>()
            .add_event::<ServerSendMessage>()
            .add_event::<ClientSendMessage>()
            .add_event::<UpdatePopulationEv>()
            // Resources
            .init_resource::<Ip>()
            .init_resource::<GameSettings>()
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
                (move_camera, move_camera_keyboard)
                    .run_if(not(
                        in_state(GameState::TraitSelection).or(in_state(GameState::InGameMenu))
                    ))
                    .in_set(InGameSet),
            )
            // Audio
            .add_systems(Startup, setup_music_btn)
            .add_systems(OnEnter(AudioState::Sound), play_music)
            .add_systems(
                Update,
                (change_audio_event, toggle_music_keyboard, play_audio_event),
            )
            //Networking
            .add_systems(
                First,
                (
                    server_receive_message.run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                )
                    .in_set(InGameSet),
            )
            .add_systems(PreUpdate, update_population_event.in_set(InGameSet))
            .add_systems(
                Update,
                server_update
                    .run_if(resource_exists::<RenetServer>)
                    .run_if(not(in_state(AppState::Game))),
            )
            .add_systems(
                Last,
                (
                    (
                        server_send_status.run_if(on_timer(Duration::from_millis(NETWORK_TIMER))),
                        server_send_message,
                    )
                        .run_if(resource_exists::<RenetServer>),
                    (
                        client_send_status.run_if(on_timer(Duration::from_millis(NETWORK_TIMER))),
                        client_send_message,
                    )
                        .run_if(resource_exists::<RenetClient>),
                )
                    .in_set(InGameSet),
            );

        // Menu
        for state in AppState::iter().filter(|s| *s != AppState::Game) {
            app.add_systems(OnEnter(state), setup_menu)
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }
        app.add_systems(
            Update,
            update_ip.run_if(in_state(AppState::MultiPlayerMenu)),
        );

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
            .add_systems(OnEnter(GameState::Running), update_game_state)
            .add_systems(OnEnter(GameState::Paused), (pause_game, update_game_state))
            .add_systems(OnExit(GameState::Paused), unpause_game)
            .add_systems(
                OnEnter(GameState::InGameMenu),
                (setup_in_game_menu, update_game_state),
            )
            .add_systems(OnExit(GameState::InGameMenu), despawn::<MenuCmp>)
            .add_systems(
                OnEnter(GameState::TraitSelection),
                (setup_trait_selection, update_game_state),
            )
            .add_systems(
                OnExit(GameState::TraitSelection),
                (despawn::<MenuCmp>, despawn::<UiCmp>, draw_ui).chain(),
            )
            .add_systems(
                OnEnter(GameState::AfterTraitSelection),
                (setup_after_trait, update_game_state),
            )
            .add_systems(
                Update,
                after_trait_check
                    .run_if(resource_exists::<RenetServer>)
                    .run_if(in_state(GameState::AfterTraitSelection)),
            )
            .add_systems(OnExit(GameState::AfterTraitSelection), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::EndGame), setup_end_game)
            .add_systems(OnExit(GameState::EndGame), despawn::<MenuCmp>)
            .add_systems(Update, toggle_pause_keyboard.in_set(InGameSet))
            // Ants
            .add_systems(
                PreUpdate,
                (resolve_pre_action, resolve_death)
                    .chain()
                    .in_set(InRunningGameSet),
            )
            .add_systems(
                Update,
                (animate_pin, update_ant_components, update_selection_icons)
                    .in_set(InRunningOrPausedGameSet),
            )
            .add_systems(
                Update,
                (
                    queue_ants_keyboard.in_set(InRunningOrPausedGameSet),
                    (
                        check_trait_timer,
                        hatch_eggs,
                        animate_ants,
                        resolve_digging,
                        resolve_harvesting,
                        resolve_harvesting_corpse,
                        resolve_healing,
                        resolve_attack_action,
                        resolve_die_action,
                        resolve_brood_action,
                        resolve_idle_action,
                        resolve_targeted_walk_action,
                        resolve_walk_action,
                        npc_buy_ants.run_if(on_timer(Duration::from_millis(ENEMY_TIMER))),
                        spawn_enemies.run_if(on_timer(Duration::from_millis(ENEMY_TIMER))),
                        resolve_expeditions.run_if(on_timer(Duration::from_millis(ENEMY_TIMER))),
                    )
                        .in_set(InRunningGameSet),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    (update_vision, spawn_tile_event).chain().run_if(not(in_state(GameState::EndGame))),
                    select_trait_event,
                    (spawn_egg_event, despawn_ant_event, damage_event).in_set(InRunningGameSet),
                    spawn_ant_event
                        .run_if(
                            in_state(GameState::AfterTraitSelection)
                                .or(in_state(GameState::Running)),
                        )
                        .in_set(InGameSet),
                    (queue_ant_event, spawn_pin_event).in_set(InRunningOrPausedGameSet),
                ),
            );

        // Persistence
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, (load_game, save_game));
    }
}
