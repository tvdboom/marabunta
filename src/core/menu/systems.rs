use crate::core::ants::components::{Ant, AntCmp};
use crate::core::assets::WorldAssets;
use crate::core::menu::buttons::{spawn_menu_button, LobbyTextCmp, MenuBtn, MenuCmp};
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::states::GameState;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use rand::Rng;
use crate::core::player::Player;

pub fn spawn_menu_ants(
    mut player: ResMut<Player>,
    mut counter: Local<u8>,
) {
    if *counter < 20 && rand::rng().random::<f32>() < 0.1 {
        *counter += 1;
        player.queue.push(AntCmp::new(Ant::BlackAnt));
    }
}

pub fn setup_menu(
    mut commands: Commands,
    game_state: Res<State<GameState>>,
    server: Option<Res<RenetServer>>,
    assets: Local<WorldAssets>,
) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    top: Val::VMin(5.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(TITLE),
                        TextFont {
                            font: assets.font("FiraMono-Medium"),
                            font_size: 100.,
                            ..default()
                        },
                    ));
                });

            match game_state.get() {
                GameState::MainMenu => {
                    spawn_menu_button(parent, MenuBtn::Play, &assets);
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        spawn_menu_button(parent, MenuBtn::Multiplayer, &assets);
                        spawn_menu_button(parent, MenuBtn::Quit, &assets);
                    }
                }
                GameState::MultiPlayerMenu => {
                    spawn_menu_button(parent, MenuBtn::HostGame, &assets);
                    spawn_menu_button(parent, MenuBtn::FindGame, &assets);
                    spawn_menu_button(parent, MenuBtn::Quit, &assets);
                }
                GameState::Lobby | GameState::ConnectedLobby => {
                    if let Some(server) = server {
                        let n_players = server.clients_id().len() + 1;

                        parent.spawn((
                            add_text(
                                if n_players == 1 {
                                    "Waiting for other players to join...".to_string()
                                } else {
                                    format!("There are {} players in the lobby...", n_players)
                                },
                                &assets,
                            ),
                            LobbyTextCmp,
                        ));

                        if n_players > 1 {
                            spawn_menu_button(parent, MenuBtn::Play, &assets);
                        }
                    } else {
                        parent.spawn((add_text("Searching for a game...", &assets), LobbyTextCmp));
                    }

                    spawn_menu_button(parent, MenuBtn::BackToMenu, &assets);
                }
                _ => (),
            }

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(5.),
                    bottom: Val::Px(5.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Created by Mavs"),
                        TextFont {
                            font: assets.font("FiraMono-Medium"),
                            font_size: 20.,
                            ..default()
                        },
                    ));
                });
        });
}
