use crate::core::ants::components::{Animation, AnimationCmp, Ant, AntCmp};
use crate::core::ants::events::QueueAntEv;
use crate::core::assets::WorldAssets;
use crate::core::constants::MAX_QUEUE_LENGTH;
use crate::core::map::systems::MapCmp;
use crate::core::map::ui::utils::{add_text, despawn_ui};
use crate::core::player::Player;
use crate::core::traits::TraitCmp;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use strum::IntoEnumIterator;

#[derive(Component)]
pub struct UiCmp;

#[derive(Component)]
pub struct FoodLabelCmp;

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
    i: usize,
) -> impl FnMut(Trigger<Pointer<Over>>, Commands, Local<WorldAssets>, Single<&Window>) {
    move |_, mut commands: Commands, assets: Local<WorldAssets>, window: Single<&Window>| {
        commands
            .spawn((
                Node {
                    top: Val::Percent(25. + 10.5 * i as f32),
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
                            ("Price", ant.price),
                            ("Health", ant.max_health),
                            ("Speed", ant.speed),
                            ("Damage", ant.damage),
                            ("Hatch time", ant.hatch_time),
                            ("Carry capacity", ant.max_carry),
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

                parent.spawn(add_text(&ant.description, "medium", 8., &assets, &window));
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
    player: Res<Player>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((
            Node {
                top: Val::Percent(3.),
                left: Val::Percent(2.),
                width: Val::Percent(6.),
                position_type: PositionType::Absolute,
                ..default()
            },
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(60.),
                    margin: UiRect::ZERO.with_right(Val::Percent(20.)),
                    ..default()
                },
                ImageNode::new(assets.image("leaf-ui")),
            ));

            parent.spawn((
                Node {
                    align_self: AlignSelf::Center,
                    ..default()
                },
                add_text(format!("{:.0}", player.food), "bold", 25., &assets, &window),
                FoodLabelCmp,
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
            for (i, ant) in ants.iter().enumerate() {
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
                    .observe(ant_hover_info_panel(ant_c.clone(), i))
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
                                    add_text(
                                        format!("{}", player.colony.get(ant).unwrap_or(&0)),
                                        "bold",
                                        10.,
                                        &assets,
                                        &window,
                                    ),
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
    mut food_q: Query<&mut Text, With<FoodLabelCmp>>,
    mut colony_q: Query<(&mut Text, &ColonyLabelCmp), Without<FoodLabelCmp>>,
    mut larva_q: Query<&mut Visibility, With<QueueLarvaCmp>>,
    mut queue_q: Query<
        (&mut Visibility, &mut ImageNode, &mut QueueButtonCmp),
        Without<QueueLarvaCmp>,
    >,
    player: Res<Player>,
    assets: Local<WorldAssets>,
) {
    // Update the food label
    food_q.get_single_mut().unwrap().0 = format!("{:.0}", player.food);

    // Update the colony labels
    for (mut text, colony) in colony_q.iter_mut() {
        text.0 = format!("{}", player.colony.get(&colony.0).unwrap_or(&0));
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
    click: Trigger<Pointer<Click>>,
    btn_q: Query<&ColonyButtonCmp>,
    mut queue_ant_ev: EventWriter<QueueAntEv>,
) {
    queue_ant_ev.send(QueueAntEv {
        ant: btn_q.get(click.entity()).unwrap().0.clone(),
    });
}

pub fn on_click_queue_button(
    click: Trigger<Pointer<Click>>,
    btn_q: Query<&QueueButtonCmp>,
    mut player: ResMut<Player>,
) {
    if let Ok(e) = btn_q.get(click.entity()) {
        if let Some(ant) = player.queue.get(e.0) {
            player.food += AntCmp::new(ant, &player).price;
            player.queue.remove(e.0);
        }
    }
}
