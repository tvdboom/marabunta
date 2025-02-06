use crate::core::states::MusicState;
use bevy::input::ButtonInput;
use bevy::prelude::*;

pub fn keys_listener(
    keyboard: Res<ButtonInput<KeyCode>>,
    music_state: Res<State<MusicState>>,
    mut next_music_state: ResMut<NextState<MusicState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        match music_state.get() {
            MusicState::Playing => next_music_state.set(MusicState::Stopped),
            MusicState::Stopped => next_music_state.set(MusicState::Playing),
        }
    }
}
