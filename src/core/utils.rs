use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct NoRotationChildCmp;

#[derive(Component)]
pub struct NoRotationParentCmp;

/// Scale a Duration by a factor
pub fn scale_duration(duration: Duration, scale: f32) -> Duration {
    let sec = (duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9) * scale;
    Duration::new(sec.trunc() as u64, (sec.fract() * 1e9) as u32)
}

/// AABB collision detection
pub fn collision(pos1: &Vec3, size1: &Vec2, pos2: &Vec3, size2: &Vec2) -> bool {
    let p1_min = pos1 - Vec3::new(size1.x / 3.0, size1.y / 3.0, 0.0);
    let p1_max = pos1 + Vec3::new(size1.x / 3.0, size1.y / 3.0, 0.0);

    let p2_min = pos2 - Vec3::new(size2.x / 3.0, size2.y / 3.0, 0.0);
    let p2_max = pos2 + Vec3::new(size2.x / 3.0, size2.y / 3.0, 0.0);

    p1_max.x > p2_min.x && p1_min.x < p2_max.x && p1_max.y > p2_min.y && p1_min.y < p2_max.y
}

/// Generic system that despawns all entities with a specific component
pub fn despawn<T: Component>(mut commands: Commands, query_c: Query<Entity, With<T>>) {
    for entity in &query_c {
        commands.entity(entity).try_despawn_recursive();
    }
}

/// Update the transform of children entities that shouldn't inherit the parent's rotation
pub fn update_transform_no_rotation(
    mut child_q: Query<(&Parent, &mut Transform), With<NoRotationChildCmp>>,
    parent_q: Query<&Transform, (With<NoRotationParentCmp>, Without<NoRotationChildCmp>)>,
) {
    for (parent, mut transform) in child_q.iter_mut() {
        if let Ok(parent_transform) = parent_q.get(parent.get()) {
            transform.rotation = parent_transform.rotation.inverse();
        }
    }
}
