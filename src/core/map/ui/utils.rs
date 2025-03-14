use crate::core::assets::WorldAssets;
use bevy::prelude::*;
use std::fmt::Debug;

#[derive(Component)]
pub struct TextSize(pub f32);

/// Change the background color of an entity
pub fn recolor<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(Trigger<E>, Query<&mut BackgroundColor>) {
    move |ev, mut bgcolor_q| {
        if let Ok(mut bgcolor) = bgcolor_q.get_mut(ev.entity()) {
            bgcolor.0 = color;
        };
    }
}

/// Despawn all entities with a specific component
pub fn despawn_ui<E: Debug + Clone + Reflect, T: Component>(
) -> impl Fn(Trigger<E>, Commands, Query<Entity, With<T>>) {
    move |_, mut commands: Commands, query_c: Query<Entity, With<T>>| {
        for entity in &query_c {
            commands.entity(entity).try_despawn_recursive();
        }
    }
}

/// Add a root UI node that covers the whole screen
pub fn add_root_node() -> (Node, PickingBehavior) {
    (
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
        },
        PickingBehavior::IGNORE, // Ignore picking to not block others
    )
}

/// Add a standard text component
pub fn add_text(
    text: impl Into<String>,
    font: &str,
    font_size: f32,
    assets: &WorldAssets,
    window: &Window,
) -> (Text, TextFont, TextSize) {
    (
        Text::new(text),
        TextFont {
            font: assets.font(font),
            font_size: font_size * window.width() / window.height(),
            ..default()
        },
        TextSize(font_size),
    )
}
