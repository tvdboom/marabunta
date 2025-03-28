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

/// Get the size of a sprite
pub fn get_sprite_size(
    transform: &Transform,
    sprite: &Sprite,
    images: &Assets<Image>,
    atlases: &Assets<TextureAtlasLayout>,
) -> Vec2 {
    sprite.custom_size.unwrap_or_else(|| {
        let size = if let Some(atlas) = &sprite.texture_atlas {
            let texture = atlas.texture_rect(atlases).unwrap();
            texture.max - texture.min
        } else {
            images.get(&sprite.image).unwrap().size()
        };

        size.as_vec2() * transform.scale.truncate()
    })
}

/// AABB collision detection from positions and sizes
pub fn collision_aabb(pos1: &Vec3, size1: &Vec2, pos2: &Vec3, size2: &Vec2) -> bool {
    let p1_min = pos1 - Vec3::new(size1.x / 4., size1.y / 4., 0.);
    let p1_max = pos1 + Vec3::new(size1.x / 4., size1.y / 4., 0.);

    let p2_min = pos2 - Vec3::new(size2.x / 4., size2.y / 4., 0.);
    let p2_max = pos2 + Vec3::new(size2.x / 4., size2.y / 4., 0.);

    p1_max.x > p2_min.x && p1_min.x < p2_max.x && p1_max.y > p2_min.y && p1_min.y < p2_max.y
}

/// AABB collision detection from sprites
pub fn collision(
    e1: (&Transform, &Sprite),
    e2: (&Transform, &Sprite),
    images: &Assets<Image>,
    atlases: &Assets<TextureAtlasLayout>,
) -> bool {
    let size1 = get_sprite_size(e1.0, e1.1, images, atlases);
    let size2 = get_sprite_size(e2.0, e2.1, images, atlases);

    collision_aabb(&e1.0.translation, &size1, &e2.0.translation, &size2)
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
