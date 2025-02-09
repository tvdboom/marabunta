use crate::core::map::map::Map;
use crate::core::map::tile::Tile;
use bevy::input::mouse::MouseWheel;
use bevy::{prelude::*, window::WindowResized};

pub const MIN_ZOOM: f32 = 0.3;
pub const MAX_ZOOM: f32 = 1.;
pub const ZOOM_FACTOR: f32 = 1.1;

#[derive(Component)]
pub struct MainCamera {
    last_width: f32,
    last_height: f32,
}

pub fn setup_camera(mut commands: Commands, window: Query<&Window>) {
    let window = window.get_single().unwrap();

    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        MainCamera {
            last_width: window.width(),
            last_height: window.height(),
        },
    ));
}

pub fn resize_camera(
    mut camera_q: Query<(&mut OrthographicProjection, &mut MainCamera)>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for ev in resize_reader.read() {
        let (mut projection, mut camera) = camera_q.single_mut();

        if ev.width != camera.last_width || ev.height != camera.last_height {
            let scale_factor_x = ev.width / camera.last_width;
            let scale_factor_y = ev.height / camera.last_height;
            let scale_factor = (scale_factor_x + scale_factor_y) / 2.0;

            projection.scale *= scale_factor;
            camera.last_width = ev.width;
            camera.last_height = ev.height;
        }
    }
}

pub fn zoom_on_scroll(
    mut camera_q: Query<
        (
            &Camera,
            &GlobalTransform,
            &mut Transform,
            &mut OrthographicProjection,
        ),
        With<MainCamera>,
    >,
    mut scroll_ev: EventReader<MouseWheel>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let (camera, global_t, mut camera_t, mut projection) = camera_q.single_mut();

    for ev in scroll_ev.read() {
        // Get cursor position in window space
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert to world space
            if let Ok(world_pos) = camera.viewport_to_world_2d(global_t, cursor_pos) {
                let scale_change = if ev.y > 0. {
                    1. / ZOOM_FACTOR
                } else {
                    ZOOM_FACTOR
                };

                let new_scale = (projection.scale * scale_change).clamp(MIN_ZOOM, MAX_ZOOM);

                // Adjust camera position to keep focus on cursor
                let shift =
                    (world_pos - camera_t.translation.truncate()) * (1. - new_scale / projection.scale);
                camera_t.translation += shift.extend(0.);

                projection.scale = new_scale;

                // Clamp camera position to stay inside the map
                // let map_size = Vec2::new(Map::SIZE.x as f32, Map::SIZE.y as f32) * Tile::SIZE;
                // let half_viewport_size =
                //     (Vec2::new(window.width(), window.height()) * camera_t.scale.truncate()) * 0.5;
                // let min_bound = half_viewport_size - map_size * 0.5;
                // let max_bound = map_size * 0.5 - half_viewport_size;
                //
                // camera_t.translation.x = camera_t.translation.x.clamp(min_bound.x, max_bound.x);
                // camera_t.translation.y = camera_t.translation.y.clamp(min_bound.y, max_bound.y);
            }
        }
    }

    // clamp_camera(&mut transform, projection.scale, window);
}
