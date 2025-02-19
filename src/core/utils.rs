use bevy::prelude::*;
use std::time::Duration;

/// Scale a Duration by a factor
pub fn scale_duration(duration: Duration, scale: f32) -> Duration {
    let sec = (duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9) * scale;
    Duration::new(sec.trunc() as u64, (sec.fract() * 1e9) as u32)
}

/// Generic system that despawns all entities with a specific component
pub fn despawn<T: Component>(mut commands: Commands, component: Query<Entity, With<T>>) {
    for entity in &component {
        commands.entity(entity).despawn_recursive();
    }
}
