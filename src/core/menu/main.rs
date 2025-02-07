use crate::core::assets::WorldAssets;
use crate::core::menu::constants::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::network::{new_renet_client, new_renet_server};
use crate::core::states::GameState;
use crate::utils::NameFromEnum;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::netcode::{NetcodeClientTransport, NetcodeServerTransport};
use bevy_renet::renet::{RenetClient, RenetServer};
use std::fmt::Debug;

#[derive(Component)]
pub struct MenuComponent;

#[derive(Component, Clone, Debug)]
pub enum MenuBtn {
    HostGame,
    FindGame,
    Play,
    BackToMenu,
    Quit,
}

fn on_click_menu_button(
    click: Trigger<Pointer<Click>>,
    btn_q: Query<&MenuBtn>,
    mut commands: Commands,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    match btn_q.get(click.entity()).unwrap() {
        MenuBtn::HostGame => {
            let (server, transport) = new_renet_server();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_game_state.set(GameState::Lobby);
        }
        MenuBtn::FindGame => {
            let (server, transport) = new_renet_client();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_game_state.set(GameState::Lobby);
        }
        MenuBtn::Play => {
            next_game_state.set(GameState::Game);
        }
        MenuBtn::BackToMenu => {
            commands.remove_resource::<RenetServer>();
            commands.remove_resource::<NetcodeServerTransport>();
            commands.remove_resource::<RenetClient>();
            commands.remove_resource::<NetcodeClientTransport>();

            next_game_state.set(GameState::Menu)
        }
        MenuBtn::Quit => std::process::exit(0),
    }
}

fn spawn_menu_button(parent: &mut ChildBuilder, btn: MenuBtn, assets: &Local<WorldAssets>) {
    parent
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Px(350.),
                height: Val::Px(80.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Px(15.)),
                padding: UiRect::all(Val::Px(15.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON.into()),
            btn.clone(),
        ))
        .observe(recolor::<Pointer<Over>>(HOVERED_BUTTON))
        .observe(recolor::<Pointer<Out>>(NORMAL_BUTTON))
        .observe(recolor::<Pointer<Down>>(PRESSED_BUTTON))
        .observe(recolor::<Pointer<Up>>(HOVERED_BUTTON))
        .observe(on_click_menu_button)
        .with_children(|parent| {
            parent.spawn(add_text(btn.as_string(), assets));
        });
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
                    spawn_menu_button(parent, MenuBtn::HostGame, &assets);
                    spawn_menu_button(parent, MenuBtn::FindGame, &assets);

                    #[cfg(not(target_arch = "wasm32"))]
                    spawn_menu_button(parent, MenuBtn::Quit, &assets);
                }
                GameState::Lobby => {
                    if server.is_some() {
                        let n_players = server.unwrap().clients_id().len();
                        parent.spawn(add_text(
                            format!("There are {n_players} players in the lobby..."),
                            &assets,
                        ));

                        if n_players > 0 {
                            spawn_menu_button(parent, MenuBtn::Play, &assets);
                        }
                    } else {
                        parent.spawn(add_text(
                            "Waiting for the host to start the game...",
                            &assets,
                        ));
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
