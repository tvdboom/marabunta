use crate::core::ants::components::Ant;
use crate::core::ants::systems::spawn_ant;
use crate::core::assets::WorldAssets;
use crate::core::map::components::Map;
use crate::core::menu::buttons::{spawn_menu_button, LobbyTextCmp, MenuBtn, MenuCmp};
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::states::GameState;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use rand::Rng;

pub fn spawn_menu_ants(
    mut commands: Commands,
    map: Res<Map>,
    mut counter: Local<u8>,
    assets: Local<WorldAssets>,
) {
    if *counter < 20 && rand::rng().random::<f32>() < 0.1 {
        *counter += 1;
        spawn_ant(
            &mut commands,
            Ant::BlackAnt,
            map.get_tile_coord(64),
            &assets,
        );
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
                GameState::Menu => {
                    spawn_menu_button(parent, MenuBtn::HostGame, &assets);
                    spawn_menu_button(parent, MenuBtn::FindGame, &assets);

                    #[cfg(not(target_arch = "wasm32"))]
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
