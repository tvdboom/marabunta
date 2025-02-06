use crate::core::menu::constants::*;
use bevy::prelude::*;

/// Generic system that takes a component as a parameter, and despawns all entities with that component
pub fn despawn_menu<T: Component>(component: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &component {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn create_root_node() -> Node {
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

pub fn create_button_node() -> (Node, BackgroundColor) {
    (
        Node {
            display: Display::Block,
            width: Val::Px(350.),
            height: Val::Px(80.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(15.)),
            padding: UiRect::all(Val::Px(15.)),
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON.into()),
    )
}

pub fn create_button_text(text: String, asset_server: &Res<AssetServer>) -> (Text, TextFont) {
    (
        Text::new(text),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 40.,
            ..default()
        },
    )
}
