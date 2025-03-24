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
    Settings,
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
