use crate::core::assets::WorldAssets;
use crate::core::constants::BUTTON_TEXT_SIZE;
use crate::core::map::ui::utils::{add_root_node, add_text};
use crate::core::menu::buttons::{spawn_menu_button, LobbyTextCmp, MenuBtn, MenuCmp};
use crate::core::player::Player;
use crate::core::states::AppState;
use crate::core::traits::{Trait, TraitCmp, TraitSelectedEv};
use crate::utils::NameFromEnum;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use rand::prelude::IteratorRandom;
use rand::rng;
use strum::IntoEnumIterator;

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

pub fn select_trait(t: Trait) -> impl FnMut(Trigger<Pointer<Click>>, EventWriter<TraitSelectedEv>) {
    move |_, mut trait_selected_ev: EventWriter<TraitSelectedEv>| {
        trait_selected_ev.send(TraitSelectedEv(t));
    }
}

pub fn setup_trait_selection(
    mut commands: Commands,
    player: Res<Player>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    let traits = Trait::iter()
        .filter(|t| !player.has_trait(&t))
        .choose_multiple(&mut rng(), 3);

    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    top: Val::Percent(5.),
                    position_type: PositionType::Absolute,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(add_text("Choose a trait", "bold", 25., &assets, &window));
                });

            parent
                .spawn(Node {
                    top: Val::Percent(12.),
                    width: Val::Percent(90.),
                    height: Val::Percent(90.),
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::ZERO.with_top(Val::Percent(5.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    for t in traits.iter() {
                        let trait_c = TraitCmp::new(t);

                        parent
                            .spawn(Node {
                                width: Val::Percent(20.),
                                height: Val::Percent(30.),
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::ZERO
                                    .with_left(Val::Percent(3.))
                                    .with_right(Val::Percent(3.)),
                                ..default()
                            })
                            .observe(select_trait(t.clone()))
                            .with_children(|parent| {
                                parent
                                    .spawn((Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::ZERO.with_bottom(Val::Percent(3.)),
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        parent.spawn(add_text(
                                            trait_c.kind.to_title(),
                                            "bold",
                                            15.,
                                            &assets,
                                            &window,
                                        ));
                                    });

                                parent.spawn((
                                    Node {
                                        margin: UiRect::ZERO.with_bottom(Val::Percent(5.)),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image(&trait_c.image)),
                                ));

                                parent
                                    .spawn((Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        parent.spawn(add_text(
                                            &trait_c.description,
                                            "medium",
                                            8.,
                                            &assets,
                                            &window,
                                        ));
                                    });
                            });
                    }
                });
        });
}

pub fn setup_game_over(
    mut commands: Commands,
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
}
