use crate::core::ants::components::AntCmp;
use crate::core::player::Player;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use uuid::Uuid;

#[derive(Resource, Default)]
pub struct SelectedAnts(pub HashSet<Uuid>);

#[derive(Default, PartialEq)]
pub struct SelectionBox {
    start: Vec2,
}

pub fn select_ants(
    mut gizmos: Gizmos,
    ant_q: Query<(&Transform, &AntCmp)>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
    player: Res<Player>,
    mut selected: ResMut<SelectedAnts>,
    mut sbox: Local<SelectionBox>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
) {
    if let Some(cursor) = window.cursor_position() {
        // Transform global cursor coord to world coord
        let (camera_t, camera) = camera_q.get_single().unwrap();
        let cursor = camera.viewport_to_world_2d(camera_t, cursor).unwrap();

        if mouse.just_pressed(MouseButton::Left) {
            sbox.start = cursor;
        } else if mouse.pressed(MouseButton::Left) {
            gizmos.rect_2d(
                Isometry2d::from_translation((sbox.start + cursor) / 2.),
                (cursor - sbox.start).abs(),
                Color::BLACK,
            );
        } else if mouse.just_released(MouseButton::Left) && *sbox != SelectionBox::default() {
            let min = Vec2::new(sbox.start.x.min(cursor.x), sbox.start.y.min(cursor.y));
            let max = Vec2::new(sbox.start.x.max(cursor.x), sbox.start.y.max(cursor.y));

            // CLear any selection unless ctrl is pressed
            if !keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                selected.0.clear();
            }

            for (ant_t, ant) in ant_q
                .iter()
                .filter(|(_, a)| player.controls(a) && a.health > 0.)
            {
                // Check if the ant is within the rectangle's bounds
                // or if the cursor is upon an ant
                let p1_min = pos1 - Vec3::new(size1.x / 4., size1.y / 4., 0.);
                let p1_max = pos1 + Vec3::new(size1.x / 4., size1.y / 4., 0.);

                if (ant_t.translation.x >= min.x
                    && ant_t.translation.x <= max.x
                    && ant_t.translation.y >= min.y
                    && ant_t.translation.y <= max.y) || (ant_t.)
                {
                    selected.0.insert(ant.id);
                }
            }

            *sbox = SelectionBox::default();
        }
    }
}
