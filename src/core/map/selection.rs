use crate::core::ants::components::AntCmp;
use crate::core::player::Player;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use uuid::Uuid;

#[derive(Event)]
pub struct SelectAntsEv {
    id: Uuid,
    clean: bool,
}

#[derive(Resource, Default)]
pub struct AntSelection(pub HashSet<Uuid>);

#[derive(Default, PartialEq)]
pub struct SelectionBox {
    start: Vec2,
}

pub fn select_ant_on_click(
    id: Uuid,
) -> impl FnMut(
    Trigger<Pointer<Click>>,
    Query<(&Transform, &AntCmp)>,
    Res<Player>,
    EventWriter<SelectAntsEv>,
    Local<f32>,
    Res<Time>,
    Single<(&Camera, &GlobalTransform)>,
    Single<&Window>,
) {
    move |_,
          ant_q: Query<(&Transform, &AntCmp)>,
          player: Res<Player>,
          mut select_ants_ev: EventWriter<SelectAntsEv>,
          mut last_clicked_t: Local<f32>,
          time: Res<Time>,
          camera: Single<(&Camera, &GlobalTransform)>,
          window: Single<&Window>| {
        let (camera, global_t) = camera.into_inner();
        let ant = ant_q.iter().find(|(_, a)| a.id == id).unwrap().1;

        // Only select own ants
        if player.controls(ant) && ant.health > 0. {
            // If double-clicked, select all ants of the same kind in viewport
            if time.elapsed_secs() - *last_clicked_t < 0.3 {
                ant_q
                    .iter()
                    .filter(|(t, a)| {
                        let view_pos = camera.world_to_viewport(global_t, t.translation).unwrap();

                        player.controls(a)
                            && a.health > 0.
                            && a.kind == ant.kind
                            && view_pos.x >= 0.
                            && view_pos.x <= window.width()
                            && view_pos.y >= 0.
                            && view_pos.y <= window.height()
                    })
                    .for_each(|(_, a)| {
                        select_ants_ev.send(SelectAntsEv {
                            id: a.id,
                            clean: false,
                        });
                    });
            } else {
                select_ants_ev.send(SelectAntsEv { id, clean: true });
            }
        }

        *last_clicked_t = time.elapsed_secs();
    }
}

pub fn select_ants_from_rect(
    mut gizmos: Gizmos,
    ant_q: Query<(&Transform, &AntCmp)>,
    mut select_ants_ev: EventWriter<SelectAntsEv>,
    player: Res<Player>,
    mut select: ResMut<AntSelection>,
    mut sbox: Local<SelectionBox>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, global_t) = camera.into_inner();

    // If shift is pressed, the camera moves
    if !keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        if let Some(cursor) = window.cursor_position() {
            // Transform global cursor coord to world coord
            let cursor = camera.viewport_to_world_2d(global_t, cursor).unwrap();

            if mouse.just_pressed(MouseButton::Left) {
                sbox.start = cursor;
            } else if mouse.pressed(MouseButton::Left) {
                gizmos.rect_2d(
                    Isometry2d::from_translation((sbox.start + cursor) / 2.),
                    (cursor - sbox.start).abs(),
                    Color::BLACK,
                );
            } else if mouse.just_released(MouseButton::Left) {
                // Clear any selection unless ctrl is pressed
                if !keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                    select.0.clear();
                }

                let min = Vec2::new(sbox.start.x.min(cursor.x), sbox.start.y.min(cursor.y));
                let max = Vec2::new(sbox.start.x.max(cursor.x), sbox.start.y.max(cursor.y));

                ant_q
                    .iter()
                    .filter(|(t, a)| {
                        player.controls(a)
                            && a.health > 0.
                            // Check if the ant is within the rectangle's bounds
                            && t.translation.x >= min.x
                            && t.translation.x <= max.x
                            && t.translation.y >= min.y
                            && t.translation.y <= max.y
                    })
                    .for_each(|(_, a)| {
                        select_ants_ev.send(SelectAntsEv {
                            id: a.id,
                            clean: false,
                        });
                    });

                *sbox = SelectionBox::default();
            }
        }
    }
}

pub fn select_ants_to_res(
    mut select_ant_ev: EventReader<SelectAntsEv>,
    mut selection: ResMut<AntSelection>,
) {
    for SelectAntsEv { id, clean } in select_ant_ev.read() {
        if !clean || !selection.0.remove(id) {
            selection.0.insert(*id);
        }
    }
}
