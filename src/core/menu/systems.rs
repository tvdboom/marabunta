use crate::core::ants::components::AntCmp;
use crate::core::assets::WorldAssets;
use crate::core::constants::BUTTON_TEXT_SIZE;
use crate::core::map::events::TileCmp;
use crate::core::map::ui::utils::{add_root_node, add_text};
use crate::core::menu::buttons::{spawn_menu_button, LobbyTextCmp, MenuBtn, MenuCmp};
use crate::core::states::AppState;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;

pub fn setup_menu(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    server: Option<Res<RenetServer>>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    top: Val::VMin(8.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(add_text(TITLE, "medium", 60., &assets, &window));
                });

            match app_state.get() {
                AppState::MainMenu => {
                    spawn_menu_button(parent, MenuBtn::Singleplayer, &assets, &window);
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        spawn_menu_button(parent, MenuBtn::Multiplayer, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
                    }
                }
                AppState::SinglePlayerMenu => {
                    spawn_menu_button(parent, MenuBtn::NewGame, &assets, &window);
                    #[cfg(not(target_arch = "wasm32"))]
                    spawn_menu_button(parent, MenuBtn::LoadGame, &assets, &window);
                    spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                }
                AppState::MultiPlayerMenu => {
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
                                    "Waiting for other players to join...".to_string()
                                } else {
                                    format!("There are {} players in the lobby...", n_players)
                                },
                                "bold",
                                BUTTON_TEXT_SIZE,
                                &assets,
                                &window,
                            ),
                            LobbyTextCmp,
                        ));

                        if n_players > 1 {
                            spawn_menu_button(parent, MenuBtn::Play, &assets, &window);
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
                _ => (),
            }

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(3.),
                    bottom: Val::Percent(3.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(add_text("Created by Mavs", "medium", 15., &assets, &window));
                });
        });
}

pub fn setup_in_game_menu(
    mut commands: Commands,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            spawn_menu_button(parent, MenuBtn::Continue, &assets, &window);
            #[cfg(not(target_arch = "wasm32"))]
            spawn_menu_button(parent, MenuBtn::SaveGame, &assets, &window);
            spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
        });
}

pub fn setup_game_over(
    mut commands: Commands,
    mut ant_q: Query<&mut Visibility, With<AntCmp>>,
    mut tile_q: Query<&mut Sprite, With<TileCmp>>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(assets.image("game-over")),
                Transform::from_scale(Vec3::splat(0.5)),
            ));
            spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
        });

    // Make the map visible
    tile_q.iter_mut().for_each(|mut s| {
        s.color.set_alpha(1.);
    });

    // Show all enemies on the map
    ant_q
        .iter_mut()
        .for_each(|mut v| *v = Visibility::Inherited);
}
