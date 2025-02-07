mod assets;
mod audio;
mod menu;
mod network;
mod states;
mod systems;

use crate::core::audio::{play_music, setup_music_btn, stop_music, toggle_music, ToggleMusicEv};
use crate::core::menu::main::{setup_menu, MenuComponent};
use crate::core::menu::utils::despawn_menu;
use crate::core::network::server_update;
use crate::core::states::{GameState, MusicState, PauseState};
use crate::core::systems::keys_listener;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Camera
            .add_systems(Startup, setup_camera)
            // Keyboard
            .add_systems(Update, keys_listener)
            // Audio
            .add_systems(Startup, setup_music_btn)
            .add_systems(OnEnter(MusicState::Playing), play_music)
            .add_systems(OnEnter(MusicState::Stopped), stop_music)
            .add_systems(Update, toggle_music)
            //Networking
            .add_systems(Update, server_update.run_if(resource_exists::<RenetServer>))
            // Menu
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), despawn_menu::<MenuComponent>)
            // Lobby
            .add_systems(OnEnter(GameState::Lobby), setup_menu)
            .add_systems(OnExit(GameState::Lobby), despawn_menu::<MenuComponent>)
            // States
            .init_state::<GameState>()
            .init_state::<PauseState>()
            .init_state::<MusicState>()
            //Events
            .add_event::<ToggleMusicEv>();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}
