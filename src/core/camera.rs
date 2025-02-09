use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera, MainCamera));
}

pub fn zoom_on_scroll(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else { return };

    let (mut transform, mut projection) = query.single_mut();

    let mut zoom_delta = 0.0;
    for event in scroll_events.iter() {
        zoom_delta += event.y * ZOOM_SPEED;
    }
    if zoom_delta == 0.0 {
        return;
    }

    let new_zoom = (projection.scale - zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
    let zoom_factor = new_zoom / projection.scale;

    let cursor_world = transform.compute_matrix().inverse().transform_point3(
        Vec3::new(cursor_pos.x - window.width() / 2.0, cursor_pos.y - window.height() / 2.0, 0.0)
    );

    transform.translation = (transform.translation - cursor_world) * zoom_factor + cursor_world;
    projection.scale = new_zoom;

    // clamp_camera(&mut transform, projection.scale, window);
}