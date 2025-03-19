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
    Sound,
    NoMusic,
    Mute,
}

#[derive(Resource, Default)]
pub struct PreviousStates {
    pub app_state: AppState,
    pub game_state: GameState,
}

pub fn update_previous_states(
    current_app_state: Res<State<AppState>>,
    current_game_state: Res<State<GameState>>,
    mut previous_state: ResMut<PreviousStates>,
) {
    *previous_state = PreviousStates {
        app_state: *current_app_state.get(),
        game_state: *current_game_state.get(),
    };
}
