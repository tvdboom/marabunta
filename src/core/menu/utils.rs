use crate::core::assets::WorldAssets;
use bevy::prelude::*;
use std::fmt::Debug;

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

/// Generic system that despawns all entities with a specific component
pub fn despawn_cmp<T: Component>(mut commands: Commands, component: Query<Entity, With<T>>) {
    for entity in &component {
        commands.entity(entity).despawn_recursive();
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
pub fn add_text(text: impl Into<String>, assets: &Local<WorldAssets>) -> (Text, TextFont) {
    (
        Text::new(text),
        TextFont {
            font: assets.font("FiraSans-Bold"),
            font_size: 40.,
            ..default()
        },
    )
}
