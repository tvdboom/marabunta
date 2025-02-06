use crate::core::assets::WorldAssets;
use crate::core::menu::constants::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::core::menu::utils::{add_button_node, add_button_text, add_root_node};
use crate::core::states::{GameState};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy_renet::netcode::{NetcodeClientTransport, NetcodeServerTransport};
use bevy_renet::renet::{RenetClient, RenetServer};
use crate::core::network::{new_renet_client, new_renet_server};
use crate::TITLE;

#[derive(Component)]
pub struct MenuComponent;

#[derive(Component, Debug)]
pub enum MenuBtn {
    HostGame,
    FindGame,
    Play,
    BackToMenu,
    Quit,
}

pub fn setup_menu(
    mut commands: Commands,
    game_state: Res<State<GameState>>,
    server: Option<Res<RenetServer>>,
    assets: Local<WorldAssets>,
) {
    commands
        .spawn((add_root_node(), MenuComponent))
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
                            font_size: 80.,
                            ..default()
                        },
                    ));
                });

            match game_state.get() {
                GameState::Menu => {
                    parent
                        .spawn((add_button_node(), Button, MenuBtn::HostGame))
                        .with_children(|parent| {
                            parent.spawn(add_button_text(MenuBtn::HostGame.as_string(), &assets));
                        });

                    parent
                        .spawn((add_button_node(), Button, MenuBtn::FindGame))
                        .with_children(|parent| {
                            parent.spawn(add_button_text(MenuBtn::FindGame.as_string(), &assets));
                        });

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        parent
                            .spawn((add_button_node(), Button, MenuBtn::Quit))
                            .with_children(|parent| {
                                parent.spawn(add_button_text(MenuBtn::Quit.as_string(), &assets));
                            });
                    }
                },
                GameState::Lobby => {
                    if server.is_some() {
                        let n_players = server.unwrap().clients_id().len();
                        parent.spawn(add_button_text(format!("There are {n_players} players in the lobby..."), &assets));

                        if n_players > 0 {
                            parent
                                .spawn((add_button_node(), Button, MenuBtn::Play))
                                .with_children(|parent| {
                                    parent.spawn(add_button_text(MenuBtn::Play.as_string(), &assets));
                                });
                        }
                    } else {
                        parent.spawn(add_button_text("Waiting for the host to start the game...", &assets));
                    }

                    parent
                        .spawn((add_button_node(), Button, MenuBtn::BackToMenu))
                        .with_children(|parent| {
                            parent.spawn(add_button_text(MenuBtn::BackToMenu.as_string(), &assets));
                        });
                },
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

pub fn btn_interact(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuBtn>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_q {
        *background_color = match *interaction {
            Interaction::None => NORMAL_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::Pressed => PRESSED_BUTTON.into(),
        }
    }
}

pub fn btn_listener(
    mut commands: Commands,
    interaction_q: Query<(&Interaction, &MenuBtn), Changed<Interaction>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction == Interaction::Pressed {
            match button {
                MenuBtn::HostGame => {
                    let (server, transport) = new_renet_server();
                    commands.insert_resource(server);
                    commands.insert_resource(transport);

                    next_game_state.set(GameState::Lobby);
                },
                MenuBtn::FindGame => {
                    let (server, transport) = new_renet_client();
                    commands.insert_resource(server);
                    commands.insert_resource(transport);

                    next_game_state.set(GameState::Lobby);
                },
                MenuBtn::Play => {
                    next_game_state.set(GameState::Game);
                },
                MenuBtn::BackToMenu => {
                    commands.remove_resource::<RenetServer>();
                    commands.remove_resource::<NetcodeServerTransport>();
                    commands.remove_resource::<RenetClient>();
                    commands.remove_resource::<NetcodeClientTransport>();

                    next_game_state.set(GameState::Menu)
                },
                MenuBtn::Quit => std::process::exit(0),
            }
        }
    }
}
