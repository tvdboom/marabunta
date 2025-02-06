use crate::core::assets::WorldAssets;
use crate::core::menu::constants::*;
use bevy::prelude::*;

/// Generic system that takes a component as a parameter, and despawns all entities with that component
pub fn despawn_menu<T: Component>(component: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &component {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn add_root_node() -> Node {
    Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        align_content: AlignContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

pub fn add_button_node() -> (Node, BackgroundColor) {
    (
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
    )
}

pub fn add_button_text(text: impl Into<String>, assets: &Local<WorldAssets>) -> (Text, TextFont) {
    (
        Text::new(text),
        TextFont {
            font: assets.font("FiraSans-Bold"),
            font_size: 40.,
            ..default()
        },
    )
}
