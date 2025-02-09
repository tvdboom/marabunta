use bevy::input::mouse::MouseWheel;
use bevy::{prelude::*, window::WindowResized};

pub const MIN_ZOOM: f32 = 0.01;
pub const MAX_ZOOM: f32 = 5.;
pub const ZOOM_FACTOR: f32 = 1.1;

#[derive(Component)]
pub struct MainCamera {
    last_width: f32,
    last_height: f32,
}

pub fn setup_camera(mut commands: Commands, window: Query<&Window>) {
    let window = window.get_single().unwrap();

    commands.spawn((Camera2d, IsDefaultUiCamera, MainCamera {
        last_width: window.width(),
        last_height: window.height(),
    }));
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
    mut camera_q: Query<(&Camera, &GlobalTransform, &mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    mut scroll_ev: EventReader<MouseWheel>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let (camera, global_t, mut camera_t, mut projection) = camera_q.single_mut();

    let mut scroll_delta = 0.;
    for ev in scroll_ev.read() {
        scroll_delta += ev.y;
    }

    if scroll_delta == 0.0 {
        return;
    }

    // Get cursor position in window space
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert to world space
        if let Ok(world_pos) = camera.viewport_to_world_2d(global_t, cursor_pos) {
            let scale_change = if scroll_delta > 0. {
                1. / ZOOM_FACTOR
            } else {
                ZOOM_FACTOR
            };

            let new_scale = (projection.scale * scale_change).clamp(MIN_ZOOM, MAX_ZOOM);

            // Adjust camera position to keep focus on cursor
            let shift = (world_pos - camera_t.translation.truncate()) * (1. - new_scale / projection.scale);
            camera_t.translation += shift.extend(0.0);

            // Apply the new scale
            camera_t.scale = Vec3::splat(new_scale);
        }
    }

    // let window = windows.single();
    // let (mut transform, mut projection) = camera_q.single_mut();
    //
    // if let Some(cursor_pos) = window.cursor_position() {
    //     for ev in scroll_ev.read() {
    //         if zoom.unwrap() > MIN_ZOOM || ev.y < 0. {
    //             let scroll_amount = if ev.unit == MouseScrollUnit::Line {
    //                 ev.y
    //             } else {
    //                 ev.y / 32.
    //             };
    //             *zoom.as_mut().unwrap() /= ZOOM_FACTOR.powf(scroll_amount);
    //
    //             let change = (zoom.unwrap() / transform.scale.x).powf(time.delta_secs() * 10.);
    //             transform.scale *= change;
    //
    //             // Convert to world space
    //             if let Some(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
    //                 let old_scale = projection.scale;
    //                 let new_scale = (old_scale * scale_change).clamp(0.1, 5.0); // Clamped zoom
    //
    //                 // Adjust camera position to keep focus on cursor
    //                 let shift = (world_pos - transform.translation.truncate()) * (1.0 - new_scale / old_scale);
    //                 transform.translation += shift.extend(0.0);
    //
    //                 // Apply the new scale
    //                 transform.scale = Vec3::splat(new_scale);
    //             }
    //
    //             let cursor = transform.compute_matrix().inverse().transform_point3(
    //                 Vec3::new(cursor_pos.x - window.width() / 2.0, cursor_pos.y - window.height() / 2.0, 0.0)
    //             );
    //
    //             transform.translation = cursor_pos.extend(0.);
    //         }
    //     }
    // }

    // clamp_camera(&mut transform, projection.scale, window);
}