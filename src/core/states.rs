use bevy::prelude::States;
use serde::{Deserialize, Serialize};

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
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
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AudioState {
    #[default]
    Playing,
    Stopped,
}
