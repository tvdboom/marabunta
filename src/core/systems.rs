use crate::core::states::{GameState, MusicState, PauseState};
use bevy::input::ButtonInput;
use bevy::prelude::*;

pub fn keys_listener(
    keyboard: Res<ButtonInput<KeyCode>>,
    music_state: Res<State<MusicState>>,
    game_state: Res<State<GameState>>,
    pause_state: Res<State<PauseState>>,
    mut next_music_state: ResMut<NextState<MusicState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        match music_state.get() {
            MusicState::Playing => next_music_state.set(MusicState::Stopped),
            MusicState::Stopped => next_music_state.set(MusicState::Playing),
        }
    }

    if *game_state.get() == GameState::Game {
        if keyboard.just_pressed(KeyCode::Space) {
            match pause_state.get() {
                PauseState::Running => next_pause_state.set(PauseState::Paused),
                PauseState::Paused => next_pause_state.set(PauseState::Running),
            }
        }
    }
}
