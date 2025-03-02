use crate::core::assets::WorldAssets;
use crate::core::map::systems::MapCmp;
use crate::core::player::Player;
use bevy::prelude::*;

#[derive(Component)]
pub struct FoodLabelCmp;

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
                Text::new(format!("{:.0}", player.food)),
                TextFont {
                    font: assets.font("FiraSans-Bold"),
                    font_size: 40.,
                    ..default()
                },
                FoodLabelCmp,
            ));
        });
}

pub fn update_ui(mut food_q: Query<&mut Text, With<FoodLabelCmp>>, player: Res<Player>) {
    if let Ok(mut text) = food_q.get_single_mut() {
        text.0 = format!("{:.0}", player.food);
    }
}
