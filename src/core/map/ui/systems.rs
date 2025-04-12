use crate::core::ants::components::{Animation, AnimationCmp, Ant, AntCmp};
use crate::core::ants::events::QueueAntEv;
use crate::core::assets::WorldAssets;
use crate::core::constants::{BUTTON_TEXT_SIZE, MAX_QUEUE_LENGTH, TITLE_TEXT_SIZE};
use crate::core::map::systems::MapCmp;
use crate::core::map::ui::utils::{add_root_node, add_text, despawn_ui};
use crate::core::menu::buttons::MenuCmp;
use crate::core::player::Players;
use crate::core::traits::{Trait, TraitCmp, TraitSelectedEv};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use rand::prelude::IteratorRandom;
use rand::rng;
use strum::IntoEnumIterator;
#[derive(Component)]
pub struct UiCmp;

#[derive(Component)]
pub struct LeavesLabelCmp;

#[derive(Component)]
pub struct NutrientsLabelCmp;

#[derive(Component)]
pub struct ColonyButtonCmp(pub Ant);

#[derive(Component)]
pub struct ColonyLabelCmp(pub Ant);

#[derive(Component)]
pub struct QueueLarvaCmp;

#[derive(Component)]
pub struct QueueButtonCmp(pub usize, pub Ant);

#[derive(Component)]
pub struct InfoPanelUi;

pub fn ant_hover_info_panel(
    ant: AntCmp,
    total: usize,
) -> impl FnMut(Trigger<Pointer<Over>>, Commands, Local<WorldAssets>, Single<&Window>) {
    move |_, mut commands: Commands, assets: Local<WorldAssets>, window: Single<&Window>| {
        commands
            .spawn((
                Node {
                    top: Val::Percent(25. - 5. * (total - 5) as f32),
                    left: Val::Percent(6.),
                    width: Val::Percent(20.),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Percent(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(88, 57, 39, 200)),
                BorderRadius::all(Val::Px(10.)),
                InfoPanelUi,
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::ZERO.with_bottom(Val::Percent(15.)),
                        ..default()
                    },))
                    .with_children(|parent| {
                        parent.spawn(add_text(ant.kind.to_title(), "bold", 15., &assets, &window));
                    });

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        margin: UiRect::ZERO.with_bottom(Val::Percent(5.)),
                        ..default()
                    })
                    .with_children(|parent| {
                        let attributes = [
                            ("Leaves", ant.price.leaves),
                            ("Nutrients", ant.price.nutrients),
                            ("Health", ant.max_health),
                            ("Speed", ant.speed),
                            ("Damage", ant.damage),
                            ("Hatch time", ant.hatch_time),
                            ("Carry capacity", ant.max_carry.leaves),
                        ];

                        for (k, v) in attributes.iter() {
                            // Skip default values
                            if *v > 1. && *v < f32::MAX {
                                parent.spawn((
                                    Node {
                                        margin: UiRect::ZERO.with_bottom(Val::Percent(2.)),
                                        ..default()
                                    },
                                    add_text(
                                        format!("{k}: {:.0}", v),
                                        "bold",
                                        8.,
                                        &assets,
                                        &window,
                                    ),
                                ));
                            }
                        }
                    });

                parent.spawn(add_text(ant.description(), "medium", 8., &assets, &window));
            });
    }
}

pub fn trait_hover_info_panel(
    t: TraitCmp,
    i: usize,
) -> impl FnMut(Trigger<Pointer<Over>>, Commands, Local<WorldAssets>, Single<&Window>) {
    move |_, mut commands: Commands, assets: Local<WorldAssets>, window: Single<&Window>| {
        commands
            .spawn((
                Node {
                    top: Val::Percent(15. + 10.5 * i as f32),
                    left: Val::Percent(69.),
                    width: Val::Percent(25.),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Percent(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(88, 57, 39, 200)),
                BorderRadius::all(Val::Px(10.)),
                InfoPanelUi,
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::ZERO.with_bottom(Val::Percent(15.)),
                        ..default()
                    },))
                    .with_children(|parent| {
                        parent.spawn(add_text(t.kind.to_title(), "bold", 15., &assets, &window));
                    });

                parent.spawn(add_text(&t.description, "medium", 8., &assets, &window));
            });
    }
}

pub fn draw_ui(
    mut commands: Commands,
    players: Res<Players>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    let player = players.main();

    commands
        .spawn((
            Node {
                top: Val::Percent(3.),
                left: Val::Percent(2.),
                width: Val::Percent(6.),
                position_type: PositionType::Absolute,
                ..default()
            },
            PickingBehavior::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(30.),
                    margin: UiRect::all(Val::Percent(15.)),
                    ..default()
                },
                ImageNode::new(assets.image("food")),
            ));

            parent.spawn((
                Node {
                    align_self: AlignSelf::Center,
                    margin: UiRect::ZERO.with_right(Val::Percent(10.)),
                    ..default()
                },
                add_text(
                    format!("{:.0}", player.resources.leaves),
                    "bold",
                    25.,
                    &assets,
                    &window,
                ),
                LeavesLabelCmp,
            ));

            parent.spawn((
                Node {
                    width: Val::Percent(30.),
                    margin: UiRect::all(Val::Percent(15.)),
                    ..default()
                },
                ImageNode::new(assets.image("nutrient")),
            ));

            parent.spawn((
                Node {
                    align_self: AlignSelf::Center,
                    ..default()
                },
                add_text(
                    format!("{:.0}", player.resources.nutrients),
                    "bold",
                    25.,
                    &assets,
                    &window,
                ),
                NutrientsLabelCmp,
            ));
        });

    let ants = Ant::iter()
        .filter(|a| player.has_ant(a))
        .collect::<Vec<_>>();

    commands
        .spawn((
            Node {
                left: Val::Percent(2.),
                width: Val::Percent(4.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            PickingBehavior::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            for ant in ants.iter() {
                let ant_c = AntCmp::new(&ant, &player);
                let atlas = assets.atlas(&ant_c.atlas(&Animation::Idle));

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.),
                            height: Val::Percent(10.),
                            margin: UiRect::all(Val::Percent(5.)),
                            ..default()
                        },
                        ImageNode {
                            image: atlas.image,
                            texture_atlas: Some(atlas.texture),
                            ..default()
                        },
                        AnimationCmp {
                            animation: Animation::Idle,
                            timer: Timer::from_seconds(
                                ant.interval(&Animation::Idle) * 3.,
                                TimerMode::Repeating,
                            ),
                            last_index: atlas.last_index,
                        },
                        ColonyButtonCmp(ant.clone()),
                    ))
                    .observe(on_click_colony_button)
                    .observe(ant_hover_info_panel(ant_c.clone(), ants.len()))
                    .observe(despawn_ui::<Pointer<Out>, InfoPanelUi>())
                    .with_children(|parent| {
                        parent
                            .spawn(Node {
                                top: Val::Percent(10.),
                                left: Val::Percent(60.),
                                position_type: PositionType::Absolute,
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn((
                                    add_text(format!("{}", 0), "bold", 10., &assets, &window),
                                    ColonyLabelCmp(ant.clone()),
                                ));
                            });

                        if let Some(key) = ant_c.key {
                            parent
                                .spawn(Node {
                                    bottom: Val::Percent(10.),
                                    left: Val::Percent(60.),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(add_text(
                                        key.to_name().chars().last().unwrap().to_string(),
                                        "bold",
                                        10.,
                                        &assets,
                                        &window,
                                    ));
                                });
                        }
                    });
            }
        });

    commands
        .spawn((
            Node {
                bottom: Val::Percent(5.),
                width: Val::Percent(100.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(1.),
                    ..default()
                },
                ImageNode {
                    image: assets.image("larva2"),
                    ..default()
                },
                QueueLarvaCmp,
            ));

            for i in 0..MAX_QUEUE_LENGTH {
                let ant_c = AntCmp::new(player.queue.get(0).unwrap_or(&Ant::Worker), &player);
                let atlas = assets.atlas(&ant_c.atlas(&Animation::Idle));

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(2.),
                            margin: UiRect::ZERO.with_left(Val::Percent(1.)),
                            ..default()
                        },
                        ImageNode {
                            image: atlas.image,
                            texture_atlas: Some(atlas.texture),
                            ..default()
                        },
                        AnimationCmp {
                            animation: Animation::Idle,
                            timer: Timer::from_seconds(
                                ant_c.kind.interval(&Animation::Idle) * 3.,
                                TimerMode::Repeating,
                            ),
                            last_index: atlas.last_index,
                        },
                        QueueButtonCmp(i, ant_c.kind.clone()),
                    ))
                    .observe(on_click_queue_button);
            }
        });

    commands
        .spawn((
            Node {
                top: Val::Percent(15.),
                right: Val::Percent(0.),
                width: Val::Percent(5.),
                height: Val::Percent(90.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            for (i, t) in player.traits.iter().enumerate() {
                let trait_c = TraitCmp::new(t);

                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Percent(10.),
                        margin: UiRect::ZERO.with_bottom(Val::Percent(15.)),
                        ..default()
                    })
                    .observe(trait_hover_info_panel(trait_c.clone(), i))
                    .observe(despawn_ui::<Pointer<Out>, InfoPanelUi>())
                    .with_children(|parent| {
                        parent.spawn(ImageNode::new(assets.image(&trait_c.image)));
                    });
            }
        });
}

pub fn animate_ui(mut animation_q: Query<(&mut AnimationCmp, &mut ImageNode)>, time: Res<Time>) {
    for (mut animation, mut image) in animation_q.iter_mut() {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if let Some(atlas) = &mut image.texture_atlas {
                atlas.index = if atlas.index == animation.last_index {
                    0
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

pub fn update_ui(
    ant_q: Query<&AntCmp>,
    mut leaves_q: Query<&mut Text, With<LeavesLabelCmp>>,
    mut nutrients_q: Query<&mut Text, (With<NutrientsLabelCmp>, Without<LeavesLabelCmp>)>,
    mut colony_q: Query<
        (&mut Text, &ColonyLabelCmp),
        (Without<LeavesLabelCmp>, Without<NutrientsLabelCmp>),
    >,
    mut larva_q: Query<&mut Visibility, With<QueueLarvaCmp>>,
    mut queue_q: Query<
        (&mut Visibility, &mut ImageNode, &mut QueueButtonCmp),
        Without<QueueLarvaCmp>,
    >,
    players: Res<Players>,
    assets: Local<WorldAssets>,
) {
    let player = players.main();

    // Update the resource labels
    leaves_q.get_single_mut().unwrap().0 = format!("{:.0}", player.resources.leaves);
    nutrients_q.get_single_mut().unwrap().0 = format!("{:.0}", player.resources.nutrients);

    // Update the colony labels
    for (mut text, colony) in colony_q.iter_mut() {
        text.0 = format!(
            "{}",
            ant_q
                .iter()
                .filter(|a| a.kind == colony.0 && a.team == players.main_id() && a.health > 0.)
                .count()
        );
    }

    // Hide the larva if the queue is empty
    if player.queue.is_empty() {
        *larva_q.get_single_mut().unwrap() = Visibility::Hidden;
    } else {
        *larva_q.get_single_mut().unwrap() = Visibility::Inherited;
    }

    // Update queue ants
    for (mut ant_v, mut image, mut button) in queue_q.iter_mut() {
        if let Some(ant) = player.queue.get(button.0) {
            *ant_v = Visibility::Inherited;

            // Only replace it if it's a different ant
            if *ant != button.1 {
                button.1 = ant.clone();

                let ant_c = AntCmp::new(ant, &player);
                let atlas = assets.atlas(&ant_c.atlas(&Animation::Idle));
                image.image = atlas.image;
                image.texture_atlas = Some(atlas.texture);
            }
        } else {
            *ant_v = Visibility::Hidden;
        }
    }
}

pub fn on_click_colony_button(
    trigger: Trigger<Pointer<Click>>,
    btn_q: Query<&ColonyButtonCmp>,
    players: Res<Players>,
    mut queue_ant_ev: EventWriter<QueueAntEv>,
) {
    queue_ant_ev.send(QueueAntEv {
        id: players.main_id(),
        ant: btn_q.get(trigger.entity()).unwrap().0.clone(),
    });
}

pub fn on_click_queue_button(
    trigger: Trigger<Pointer<Click>>,
    btn_q: Query<&QueueButtonCmp>,
    mut players: ResMut<Players>,
) {
    let player = players.main_mut();

    if trigger.event.button == PointerButton::Secondary {
        if let Ok(QueueButtonCmp(i, _)) = btn_q.get(trigger.entity()) {
            if let Some(ant) = player.queue.get(*i) {
                let price = AntCmp::new(ant, &player).price;
                player.resources += price;
                player.queue.remove(*i);
            }
        }
    }
}

pub fn select_trait(t: Trait) -> impl FnMut(Trigger<Pointer<Click>>, EventWriter<TraitSelectedEv>) {
    move |trigger: Trigger<Pointer<Click>>, mut trait_selected_ev: EventWriter<TraitSelectedEv>| {
        if trigger.event.button == PointerButton::Primary {
            trait_selected_ev.send(TraitSelectedEv {
                selected: t.clone(),
            });
        }
    }
}

pub fn setup_trait_selection(
    mut commands: Commands,
    players: Res<Players>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    let player = players.main();

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
                    parent.spawn(add_text(
                        "Choose a trait",
                        "bold",
                        TITLE_TEXT_SIZE,
                        &assets,
                        &window,
                    ));
                });

            parent
                .spawn(Node {
                    top: Val::Percent(12.),
                    width: Val::Percent(100.),
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
                                    .with_left(Val::Percent(1.))
                                    .with_right(Val::Percent(1.)),
                                ..default()
                            })
                            .observe(select_trait(t.clone()))
                            .with_children(|parent| {
                                parent
                                    .spawn(Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::ZERO.with_bottom(Val::Percent(3.)),
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        parent.spawn(add_text(
                                            trait_c.kind.to_title(),
                                            "bold",
                                            15.,
                                            &assets,
                                            &window,
                                        ));
                                    });

                                parent
                                    .spawn(Node {
                                        width: Val::Percent(100.),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Node {
                                                width: Val::Percent(100.),
                                                height: Val::Percent(100.),
                                                ..default()
                                            },
                                            ImageNode::new(assets.image(&trait_c.image)),
                                        ));

                                        parent
                                            .spawn((
                                                Node {
                                                    bottom: Val::Percent(2.),
                                                    width: Val::Percent(90.),
                                                    position_type: PositionType::Absolute,
                                                    flex_direction: FlexDirection::ColumnReverse,
                                                    margin: UiRect::all(Val::Percent(3.)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgba(0., 0., 0., 0.9)),
                                                BorderRadius::all(Val::Px(10.)),
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    Node {
                                                        margin: UiRect::all(Val::Percent(3.)),
                                                        ..default()
                                                    },
                                                    add_text(
                                                        &trait_c.description,
                                                        "medium",
                                                        8.,
                                                        &assets,
                                                        &window,
                                                    ),
                                                ));
                                            });
                                    });
                            });
                    }
                });
        });
}

pub fn setup_after_trait(
    mut commands: Commands,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((add_root_node(), MenuCmp))
        .with_children(|parent| {
            parent.spawn(add_text(
                "Waiting for other players to select a trait...".to_string(),
                "bold",
                BUTTON_TEXT_SIZE,
                &assets,
                &window,
            ));
        });
}
