use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(States, EnumIter, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    SinglePlayerMenu,
    MultiPlayerMenu,
    Lobby,
    ConnectedLobby,
    Game,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum GameState {
    #[default]
    Running,
    Paused,
    InGameMenu,
    TraitSelection,
    GameOver,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum AudioState {
    #[default]
    Playing,
    Stopped,
}

#[derive(Resource, Default)]
pub struct PreviousGameState(pub GameState);

pub fn update_previous_game_state(
    current_state: Res<State<GameState>>,
    mut previous_game_state: ResMut<PreviousGameState>,
) {
    if *current_state.get() != previous_game_state.0 {
        previous_game_state.0 = *current_state.get();
    }
}
