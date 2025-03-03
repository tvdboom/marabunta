use crate::core::ants::components::{Animation, AnimationCmp, Ant, AntCmp};
use crate::core::assets::WorldAssets;
use crate::core::map::systems::MapCmp;
use crate::core::map::ui::utils::add_text;
use crate::core::player::Player;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use strum::IntoEnumIterator;
use crate::core::ants::events::QueueAntEv;

#[derive(Component)]
pub struct FoodLabelCmp;

#[derive(Component)]
pub struct ColonyButtonCmp(pub Ant);

#[derive(Component)]
pub struct ColonyLabelCmp(pub Ant);

pub fn draw_ui(mut commands: Commands, player: Res<Player>, assets: Local<WorldAssets>) {
    commands
        .spawn((
            Node {
                width: Val::Px(150.),
                height: Val::Px(50.),
                top: Val::Px(50.),
                left: Val::Px(50.),
                position_type: PositionType::Absolute,
                ..default()
            },
            PickingBehavior::IGNORE,
            MapCmp,
        ))
        .with_children(|parent| {
            parent.spawn(ImageNode::new(assets.image("leaf1")));
            parent.spawn((
                add_text(format!("{:.0}", player.food), 40., &assets),
                FoodLabelCmp,
            ));
        });

    commands
        .spawn((
            Node {
                top: Val::Px(150.),
                left: Val::Px(50.),
                width: Val::Px(50.),
                height: Val::Px(500.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            PickingBehavior::IGNORE,
            MapCmp,
        ))
        .with_children(|parent| {
            for (i, ant) in Ant::iter().enumerate() {
                let ant_c = AntCmp::new(&ant, player.id);
                let scale = match i {
                    0..3 => 1.,
                    _ => 1.2,
                };

                let atlas = assets.atlas(&format!(
                    "{}_{}",
                    ant.to_snake(),
                    Animation::Idle.to_snake()
                ));

                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Percent(28.),
                        position_type: PositionType::Relative,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::ZERO.with_bottom(Val::Px((i / 3) as f32 * 8.)),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                Node {
                                    width: Val::Percent(100.),
                                    height: Val::Percent(100.),
                                    ..default()
                                },
                                ImageNode {
                                    image: atlas.image,
                                    texture_atlas: Some(atlas.texture),
                                    ..default()
                                },
                                Transform::from_scale(Vec3::splat(scale)),
                                AnimationCmp {
                                    animation: Animation::Idle,
                                    timer: Timer::from_seconds(
                                        Animation::Idle.interval() * 3.,
                                        TimerMode::Repeating,
                                    ),
                                    last_index: atlas.last_index,
                                },
                                ColonyButtonCmp(ant.clone()),
                            ))
                            .observe(on_click_ui_button)
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
                                                format!(
                                                    "{}",
                                                    player.colony.get(&ant).unwrap_or(&0)
                                                ),
                                                30.,
                                                &assets,
                                            ),
                                            Transform::from_scale(Vec3::splat(1. / scale)),
                                            ColonyLabelCmp(ant),
                                        ));
                                    });
                            });

                        if let Some(key) = ant_c.key {
                            parent
                                .spawn(Node {
                                    bottom: Val::Percent(0.),
                                    left: Val::Percent(60.),
                                    position_type: PositionType::Absolute,
                                    align_content: AlignContent::Center,
                                    align_items: AlignItems::Center,
                                    align_self: AlignSelf::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((add_text(
                                        match key {
                                            KeyCode::KeyZ => "Z",
                                            KeyCode::KeyX => "X",
                                            KeyCode::KeyC => "C",
                                            KeyCode::KeyV => "V",
                                            KeyCode::KeyB => "B",
                                            _ => unreachable!(),
                                        },
                                        20.,
                                        &assets,
                                    ),));
                                });
                        }
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
    player: Res<Player>,
) {
    food_q.get_single_mut().unwrap().0 = format!("{:.0}", player.food);

    for (mut text, colony) in colony_q.iter_mut() {
        text.0 = format!("{}", player.colony.get(&colony.0).unwrap_or(&0));
    }
}

pub fn on_click_ui_button(
    click: Trigger<Pointer<Click>>,
    btn_q: Query<&ColonyButtonCmp>,
    mut queue_ant_ev: EventWriter<QueueAntEv>,
) {
    queue_ant_ev.send(QueueAntEv { ant: btn_q.get(click.entity()).unwrap().0.clone() });
}
