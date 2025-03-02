use crate::core::ants::components::Ant;
use crate::core::assets::WorldAssets;
use crate::core::map::systems::MapCmp;
use crate::core::map::ui::utils::add_text;
use crate::core::player::Player;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use strum::IntoEnumIterator;

#[derive(Component)]
pub struct FoodLabelCmp;

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
                height: Val::Px(250.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            PickingBehavior::IGNORE,
            MapCmp,
        ))
        .with_children(|parent| {
            for ant in Ant::iter() {
                parent.spawn((
                           Node {
                               top: Val::Px(0.),
                               left: Val::Px(0.),
                               width: Val::Px(50.),
                               height: Val::Px(250.),
                               position_type: PositionType::Absolute,
                               flex_direction: FlexDirection::Row,
                               ..default()
                           },
                       ))
                    .with_children(|parent| {
                        parent.spawn((
                            ImageNode::new(assets.image(&ant.to_snake())),
                            Transform::from_scale(Vec3::splat(0.5)),
                        ));
                        parent.spawn((add_text(
                            format!("{}", player.colony.get(&ant).unwrap_or(&0)),
                            20.,
                            &assets,
                        ), ColonyLabelCmp(ant)));
                    });
            }
        });
}

pub fn update_ui(
    mut food_q: Query<&mut Text, With<FoodLabelCmp>>,
    mut colony_q: Query<(&mut Text, &ColonyLabelCmp)>,
    player: Res<Player>,
) {
    food_q.get_single_mut().unwrap().0 = format!("{:.0}", player.food);

    for (mut text, colony) in colony_q.iter_mut() {
        text.0 = format!("{}", player.colony.get(&colony.0).unwrap_or(&0));
    }
}
