use crate::core::audio::ToggleMusicEv;
use crate::core::states::{GameState, PauseState};
use bevy::input::ButtonInput;
use bevy::prelude::*;

pub fn keys_listener(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    pause_state: Res<State<PauseState>>,
    mut toggle_music_ev: EventWriter<ToggleMusicEv>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        toggle_music_ev.send(ToggleMusicEv);
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
