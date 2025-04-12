use crate::core::ants::components::{Ant, AntCmp};
use crate::core::assets::WorldAssets;
use crate::core::constants::{
    BUTTON_TEXT_SIZE, DISABLED_BUTTON_COLOR, NORMAL_BUTTON_COLOR, SUBTITLE_TEXT_SIZE,
};
use crate::core::game_settings::GameSettings;
use crate::core::map::events::TileCmp;
use crate::core::map::ui::utils::{add_root_node, add_text};
use crate::core::menu::buttons::{
    spawn_menu_button, DisabledButton, IpTextCmp, LobbyTextCmp, MenuBtn, MenuCmp,
};
use crate::core::menu::settings::{spawn_label, SettingsBtn};
use crate::core::player::Players;
use crate::core::states::AppState;
use crate::utils::get_local_ip;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use std::net::IpAddr;

#[derive(Resource)]
pub struct Ip(pub String);

impl Default for Ip {
    fn default() -> Self {
        Self(get_local_ip().to_string())
    }
}

pub fn setup_menu(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    server: Option<Res<RenetServer>>,
    game_settings: Res<GameSettings>,
    ip: Res<Ip>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    top: Val::VMin(5.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(add_text(TITLE, "medium", 60., &assets, &window));
                });

            parent
                .spawn(Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::ZERO.with_top(Val::Percent(10.)),
                    ..default()
                })
                .with_children(|parent| match app_state.get() {
                    AppState::MainMenu => {
                        spawn_menu_button(parent, MenuBtn::Singleplayer, &assets, &window);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::Multiplayer, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::Settings, &assets, &window);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
                    }
                    AppState::SinglePlayerMenu => {
                        spawn_menu_button(parent, MenuBtn::NewGame, &assets, &window);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::LoadGame, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    AppState::MultiPlayerMenu => {
                        parent.spawn((
                            add_text(
                                format!("Ip: {}", ip.0),
                                "bold",
                                BUTTON_TEXT_SIZE,
                                &assets,
                                &window,
                            ),
                            IpTextCmp,
                        ));
                        spawn_menu_button(parent, MenuBtn::HostGame, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::FindGame, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    AppState::Lobby | AppState::ConnectedLobby => {
                        if let Some(server) = server {
                            let n_players = server.clients_id().len() + 1;

                            parent.spawn((
                                add_text(
                                    if n_players == 1 {
                                        format!("Waiting for other players to join {}...", get_local_ip())
                                    } else {
                                        format!("There are {n_players} players in the lobby.\nWaiting for other players to join {}...", get_local_ip())
                                    },
                                    "bold",
                                    BUTTON_TEXT_SIZE,
                                    &assets,
                                    &window,
                                ),
                                LobbyTextCmp,
                            ));

                            if n_players > 1 {
                                spawn_menu_button(parent, MenuBtn::NewGame, &assets, &window);
                                spawn_menu_button(parent, MenuBtn::LoadGame, &assets, &window);
                            }
                        } else {
                            parent.spawn((
                                add_text(
                                    "Searching for a game...",
                                    "bold",
                                    BUTTON_TEXT_SIZE,
                                    &assets,
                                    &window,
                                ),
                                LobbyTextCmp,
                            ));
                        }

                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    AppState::Settings => {
                        parent
                            .spawn((Node {
                                width: Val::Percent(40.),
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::ZERO.with_top(Val::Percent(-2.)),
                                padding: UiRect {
                                    top: Val::Percent(1.),
                                    left: Val::Percent(2.5),
                                    right: Val::Percent(2.5),
                                    bottom: Val::Percent(1.),
                                },
                                ..default()
                            },))
                            .with_children(|parent| {
                                spawn_label(
                                    parent,
                                    "Color",
                                    vec![SettingsBtn::Black, SettingsBtn::Red],
                                    &game_settings,
                                    &assets,
                                    &window,
                                );
                                spawn_label(
                                    parent,
                                    "Background",
                                    vec![SettingsBtn::Soil, SettingsBtn::Rock],
                                    &game_settings,
                                    &assets,
                                    &window,
                                );
                                spawn_label(
                                    parent,
                                    "Fog of war",
                                    vec![SettingsBtn::None, SettingsBtn::Half, SettingsBtn::Full],
                                    &game_settings,
                                    &assets,
                                    &window,
                                );
                                spawn_label(
                                    parent,
                                    "Number of opponents",
                                    vec![
                                        SettingsBtn::Zero,
                                        SettingsBtn::One,
                                        SettingsBtn::Two,
                                        SettingsBtn::Three,
                                    ],
                                    &game_settings,
                                    &assets,
                                    &window,
                                );
                                spawn_label(
                                    parent,
                                    "Audio",
                                    vec![
                                        SettingsBtn::Mute,
                                        SettingsBtn::NoMusic,
                                        SettingsBtn::Sound,
                                    ],
                                    &game_settings,
                                    &assets,
                                    &window,
                                );
                            });

                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    _ => (),
                });

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(3.),
                    bottom: Val::Percent(3.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(add_text("Created by Mavs", "medium", SUBTITLE_TEXT_SIZE, &assets, &window));
                });
        });
}

pub fn update_ip(
    mut commands: Commands,
    mut btn_q: Query<(Entity, &mut BackgroundColor, &MenuBtn)>,
    mut text_q: Query<&mut Text, With<IpTextCmp>>,
    mut ip: ResMut<Ip>,
    mut invalid_ip: Local<bool>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for key in keyboard.get_just_pressed() {
        match key {
            KeyCode::Digit0 => ip.0.push('0'),
            KeyCode::Digit1 => ip.0.push('1'),
            KeyCode::Digit2 => ip.0.push('2'),
            KeyCode::Digit3 => ip.0.push('3'),
            KeyCode::Digit4 => ip.0.push('4'),
            KeyCode::Digit5 => ip.0.push('5'),
            KeyCode::Digit6 => ip.0.push('6'),
            KeyCode::Digit7 => ip.0.push('7'),
            KeyCode::Digit8 => ip.0.push('8'),
            KeyCode::Digit9 => ip.0.push('9'),
            KeyCode::Period => ip.0.push('.'),
            KeyCode::Backspace => {
                ip.0.pop();
            }
            _ => (),
        };
    }

    for (button_e, mut bgcolor, btn) in &mut btn_q {
        if *btn == MenuBtn::FindGame {
            if ip.0.parse::<IpAddr>().is_ok() {
                // Only enable once when the ip becomes valid
                if *invalid_ip {
                    bgcolor.0 = NORMAL_BUTTON_COLOR;
                    commands.entity(button_e).remove::<DisabledButton>();
                    *invalid_ip = false;
                }
            } else {
                commands.entity(button_e).insert(DisabledButton);
                bgcolor.0 = DISABLED_BUTTON_COLOR;
                *invalid_ip = true;
            }
        }
    }

    if let Ok(mut text) = text_q.get_single_mut() {
        text.0 = format!("Ip: {}", ip.0);
    }
}

pub fn setup_in_game_menu(
    mut commands: Commands,
    players: Res<Players>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            spawn_menu_button(parent, MenuBtn::Continue, &assets, &window);
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Only the host can save a multiplayer game
                if players.main_id() == 0 {
                    spawn_menu_button(parent, MenuBtn::SaveGame, &assets, &window);
                }
            }
            spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
        });
}

pub fn setup_end_game(
    mut commands: Commands,
    mut ant_q: Query<(&mut Visibility, &AntCmp)>,
    mut tile_q: Query<&mut Sprite, With<TileCmp>>,
    players: Res<Players>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    let image = if ant_q
        .iter()
        .filter(|(_, a)| a.kind == Ant::Queen && a.team == players.main_id() && a.health > 0.)
        .count()
        == 0
    {
        "defeat"
    } else {
        "victory"
    };

    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent.spawn(ImageNode::new(assets.image(image)));
            spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
        });

    // Make the map visible
    tile_q.iter_mut().for_each(|mut s| {
        s.color.set_alpha(1.);
    });

    // Show all enemies on the map
    ant_q
        .iter_mut()
        .for_each(|(mut v, _)| *v = Visibility::Inherited);
}
