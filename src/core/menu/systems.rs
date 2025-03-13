use crate::core::assets::WorldAssets;
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

            match app_state.get() {
                AppState::MainMenu => {
                    spawn_menu_button(parent, MenuBtn::Singleplayer, &assets);
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        spawn_menu_button(parent, MenuBtn::Multiplayer, &assets);
                        spawn_menu_button(parent, MenuBtn::Quit, &assets);
                    }
                }
                AppState::SinglePlayerMenu => {
                    spawn_menu_button(parent, MenuBtn::NewGame, &assets);
                    #[cfg(not(target_arch = "wasm32"))]
                    spawn_menu_button(parent, MenuBtn::LoadGame, &assets);
                    spawn_menu_button(parent, MenuBtn::Back, &assets);
                }
                AppState::MultiPlayerMenu => {
                    spawn_menu_button(parent, MenuBtn::HostGame, &assets);
                    spawn_menu_button(parent, MenuBtn::FindGame, &assets);
                    spawn_menu_button(parent, MenuBtn::Back, &assets);
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
                                40.,
                                &assets,
                            ),
                            LobbyTextCmp,
                        ));

                        if n_players > 1 {
                            spawn_menu_button(parent, MenuBtn::Play, &assets);
                        }
                    } else {
                        parent.spawn((
                            add_text("Searching for a game...", 40., &assets),
                            LobbyTextCmp,
                        ));
                    }

                    spawn_menu_button(parent, MenuBtn::Back, &assets);
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

pub fn setup_in_game_menu(mut commands: Commands, assets: Local<WorldAssets>) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            spawn_menu_button(parent, MenuBtn::Continue, &assets);
            #[cfg(not(target_arch = "wasm32"))]
            spawn_menu_button(parent, MenuBtn::SaveGame, &assets);
            spawn_menu_button(parent, MenuBtn::Quit, &assets);
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
) {
    let traits = Trait::iter()
        .filter(|t| !player.has_trait(&t))
        .choose_multiple(&mut rng(), 3);

    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    bottom: Val::Percent(8.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(add_text("Choose a trait", 50., &assets));
                });

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    for t in traits.iter() {
                        let trait_c = TraitCmp::new(t);

                        parent
                            .spawn(Node {
                                width: Val::Px(300.),
                                height: Val::Px(500.),
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::ZERO
                                    .with_left(Val::Px(20.))
                                    .with_right(Val::Px(20.)),
                                ..default()
                            })
                            .observe(select_trait(t.clone()))
                            .with_children(|parent| {
                                parent
                                    .spawn((Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::ZERO.with_bottom(Val::Px(-30.)),
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        parent.spawn(add_text(
                                            trait_c.kind.to_title(),
                                            30.,
                                            &assets,
                                        ));
                                    });

                                parent.spawn((
                                    ImageNode::new(assets.image(&trait_c.image)),
                                    Transform::from_scale(Vec3::splat(0.8)),
                                ));

                                parent
                                    .spawn((Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::ZERO.with_top(Val::Px(-30.)),
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Text::new(&trait_c.description),
                                            TextFont {
                                                font_size: 13.,
                                                ..default()
                                            },
                                        ));
                                    });
                            });
                    }
                });
        });
}

pub fn setup_game_over(mut commands: Commands, assets: Local<WorldAssets>) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(assets.image("game-over")),
                Transform::from_scale(Vec3::splat(0.5)),
            ));
            spawn_menu_button(parent, MenuBtn::Quit, &assets);
        });
}
