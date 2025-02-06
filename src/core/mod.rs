mod assets;
mod audio;
mod menu;
mod states;
mod systems;

use crate::core::audio::{music_btn_listener, play_music, setup_music_btn, stop_music};
use crate::core::menu::main::{btn_interact, setup_main_menu, MainMenuComponent};
use crate::core::menu::utils::despawn_menu;
use crate::core::states::{GameState, MusicState, PauseState};
use crate::core::systems::keys_listener;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Camera
            .add_systems(Startup, setup_camera)
            // Keyboard
            .add_systems(Update, keys_listener)
            // Audio
            .add_systems(OnEnter(MusicState::Playing), (play_music, setup_music_btn))
            .add_systems(OnEnter(MusicState::Stopped), (stop_music, setup_music_btn))
            .add_systems(Update, music_btn_listener)
            // Menu
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(Update, btn_interact.run_if(in_state(GameState::MainMenu)))
            .add_systems(
                OnExit(GameState::MainMenu),
                despawn_menu::<MainMenuComponent>,
            )
            // States
            .init_state::<GameState>()
            .init_state::<PauseState>()
            .init_state::<MusicState>();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}
