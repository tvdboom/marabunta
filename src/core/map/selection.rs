use crate::core::ants::components::AntCmp;
use crate::core::player::Player;
use bevy::prelude::*;
use uuid::Uuid;
use crate::core::constants::MAX_Z_SCORE;

#[derive(Resource)]
pub struct SelectedAnts(pub Vec<Uuid>);

impl Default for SelectedAnts {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[derive(Component)]
pub struct SelectionBoxCmp {
    start_pos: Option<Vec2>,
    current_pos: Option<Vec2>,
}

impl SelectionBoxCmp {
    fn from(pos: Vec2) -> Self {
        Self {
            start_pos: Some(pos),
            current_pos: None,
        }
    }
}

pub fn select_ants(
    mut commands: Commands,
    ant_q: Query<(&Transform, &AntCmp)>,
    mut box_q: Query<(&mut Mesh2d, &mut SelectionBoxCmp)>,
    player: Res<Player>,
    mut selected: ResMut<SelectedAnts>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
) {
    if let Some(cursor) = window.cursor_position() {
        if mouse.pressed(MouseButton::Left) {
            if let Ok((mut mesh, mut sbox)) = box_q.get_single_mut() {
                // mesh.set


                sbox.current_pos = Some(cursor);
            } else {
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(0., 0.))),
                    MeshMaterial2d(
                        materials.add(ColorMaterial::from(Color::srgba(0., 0., 0., 0.8))),
                    ),
                    Transform::from_translation(cursor.extend(MAX_Z_SCORE)),
                    SelectionBoxCmp::from(cursor),
                    ));
            }
        } else if mouse.just_released(MouseButton::Left) {
            // Mouse released, finalize the selection box
            if let (Some(start), Some(end)) = (selection_box.start_pos, selection_box.current_pos) {
                let min = Vec2::new(start.x.min(end.x), start.y.min(end.y));
                let max = Vec2::new(start.x.max(end.x), start.y.max(end.y));

                for (transform, ant) in ant_q
                    .iter()
                    .filter(|(_, a)| player.controls(a) && a.health > 0.)
                {
                    if transform.translation.x >= min.x
                        && transform.translation.x <= max.x
                        && transform.translation.y >= min.y
                        && transform.translation.y <= max.y
                    {
                        selected.0.push(ant.id);
                    }
                }
            }

            *selection_box = SelectionBox::default();
        }
    }
}
