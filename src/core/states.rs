use bevy::prelude::States;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    MultiPlayerMenu,
    Lobby,
    ConnectedLobby,
    Game,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum MusicState {
    #[default]
    Playing,
    Stopped,
}
