use crate::core::ants::components::{Action, Ant, AntCmp, AttackCmp, Behavior, Corpse, DefendCmp};
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioEv;
use crate::core::constants::MAX_Z_SCORE;
use crate::core::map::loc::Loc;
use crate::core::map::map::Map;
use crate::core::map::systems::MapCmp;
use crate::core::map::tile::Leaf;
use crate::core::player::Players;
use crate::core::traits::Trait;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;

#[derive(Resource, Default)]
pub struct AntSelection(pub HashSet<Entity>);

#[derive(Event)]
pub struct SelectAntEv {
    entity: Entity,
    clean: bool,
}

#[derive(Event)]
pub struct PinEv(pub Loc);

#[derive(Default, PartialEq)]
pub struct SelectionBox {
    start: Vec2,
}

#[derive(Component)]
pub struct PinCmp {
    pub loc: Loc,
    pub timer: Timer,
    pub index: usize,
}

pub fn spawn_pin_event(
    mut commands: Commands,
    mut pin_ev: EventReader<PinEv>,
    assets: Local<WorldAssets>,
) {
    for PinEv(loc) in pin_ev.read() {
        commands.spawn((
            Sprite::from_image(assets.image("pin")),
            Transform {
                translation: (Map::get_coord_from_loc(loc) + Vec2::new(0., 5.)).extend(MAX_Z_SCORE),
                scale: Vec3::splat(0.03),
                ..default()
            },
            PickingBehavior::IGNORE,
            PinCmp {
                loc: loc.clone(),
                timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                index: 0,
            },
            MapCmp,
        ));
    }
}

pub fn animate_pin(mut pin_q: Query<(&mut Transform, &mut PinCmp)>, time: Res<Time>) {
    for (mut transform, mut animation) in &mut pin_q {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if animation.index / 5 % 2 == 0 {
                transform.translation.y += 1.;
            } else {
                transform.translation.y -= 1.;
            };

            animation.index += 1;
        }
    }
}

pub fn select_loc_on_click(
    trigger: Trigger<Pointer<Click>>,
    mut ant_q: Query<&mut AntCmp>,
    players: Res<Players>,
    map: Res<Map>,
    mut selection: ResMut<AntSelection>,
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let player = players.main();
    let (camera, global_t) = *camera;

    match trigger.event.button {
        PointerButton::Primary
            if !keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) =>
        {
            selection.0.clear();
        }
        PointerButton::Secondary => {
            let mut success = false;

            let cursor = camera
                .viewport_to_world_2d(global_t, window.cursor_position().unwrap())
                .unwrap();

            let loc = map.get_loc(&cursor.extend(0.));
            let tile = map.get_tile(loc.x, loc.y).unwrap();
            if tile.explored.contains(&players.main_id()) && map.is_walkable(&loc) {
                for ant_e in selection.0.iter() {
                    if let Ok(mut ant) = ant_q.get_mut(*ant_e) {
                        // Restrict the queen's movement to the base
                        if ant.kind == Ant::Queen
                            && !player.has_trait(&Trait::WanderingQueen)
                            && map
                                .get_tile(loc.x, loc.y)
                                .unwrap()
                                .base
                                .filter(|b| *b == player.id)
                                .is_none()
                        {
                            continue;
                        }

                        ant.command = Some(Behavior::ProtectLoc(loc));
                        ant.action = Action::Walk(loc);
                        success = true;
                    }
                }
            } else {
                for ant_e in selection.0.iter() {
                    if let Ok(mut ant) = ant_q.get_mut(*ant_e) {
                        if ant.kind == Ant::Excavator {
                            ant.command = Some(Behavior::Dig(loc));
                            ant.action = Action::Idle;
                            success = true;
                        }
                    }
                }
            }

            if !success {
                play_audio_ev.send(PlayAudioEv {
                    name: "error",
                    volume: 0.5,
                });
            }
        }
        _ => (),
    }
}

pub fn select_leaf_on_click(
    mut trigger: Trigger<Pointer<Click>>,
    mut ant_q: Query<&mut AntCmp>,
    leaf_q: Query<(Entity, &GlobalTransform), With<Leaf>>,
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    players: Res<Players>,
    map: Res<Map>,
    selection: Res<AntSelection>,
) {
    let player = players.main();

    if trigger.event.button == PointerButton::Secondary {
        if let Ok((leaf_e, leaf_t)) = leaf_q.get(trigger.entity()) {
            let mut success = false;

            let loc = map.get_loc(&leaf_t.translation());

            // Workers go harvest the leaf; the rest protects the location
            for ant_e in selection.0.iter() {
                if let Ok(mut sel) = ant_q.get_mut(*ant_e) {
                    if sel.kind == Ant::Queen && !player.has_trait(&Trait::WanderingQueen) {
                        continue;
                    }

                    if sel.kind == Ant::Worker {
                        sel.command = Some(Behavior::Harvest(leaf_e));
                        sel.action = Action::Walk(loc);
                        success = true;
                    } else {
                        sel.command = Some(Behavior::ProtectLoc(loc));
                        sel.action = Action::Walk(loc);
                        success = true;
                    }
                }
            }

            if !success {
                play_audio_ev.send(PlayAudioEv {
                    name: "error",
                    volume: 0.5,
                });
            }
        }
    }

    // Stop the click from reaching the tile itself
    trigger.propagate(false);
}

pub fn select_ant_on_click(
    trigger: Trigger<Pointer<Click>>,
    mut ant_q: Query<(Entity, &Transform, &mut AntCmp)>,
    players: Res<Players>,
    map: Res<Map>,
    mut select_ants_ev: EventWriter<SelectAntEv>,
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    selection: Res<AntSelection>,
    mut last_clicked_t: Local<f32>,
    time: Res<Time>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let player = players.main();
    let (camera, global_t) = *camera;

    let (ant_e, ant_t, ant) = ant_q.get(trigger.entity()).unwrap();
    let ant = ant.clone();
    let loc = map.get_loc(&ant_t.translation);

    match trigger.event.button {
        // Left mouse button used for selection
        PointerButton::Primary => {
            if ant.team == player.id && ant.health > 0. {
                // If double-clicked, select all ants of the same kind in viewport
                if time.elapsed_secs() - *last_clicked_t < 0.3 {
                    ant_q
                        .iter()
                        .filter(|(_, t, a)| {
                            let view_pos =
                                camera.world_to_viewport(global_t, t.translation).unwrap();

                            a.team == player.id
                                && a.health > 0.
                                && a.kind == ant.kind
                                && view_pos.x >= 0.
                                && view_pos.x <= window.width()
                                && view_pos.y >= 0.
                                && view_pos.y <= window.height()
                        })
                        .for_each(|(e, _, _)| {
                            select_ants_ev.send(SelectAntEv {
                                entity: e,
                                clean: false,
                            });
                        });
                } else {
                    select_ants_ev.send(SelectAntEv {
                        entity: ant_e,
                        clean: true,
                    });
                }
            }

            *last_clicked_t = time.elapsed_secs();
        }
        // Right mouse button used to set a new action
        PointerButton::Secondary => {
            let mut success = false;

            for sel_e in selection.0.iter() {
                if let Ok((_, _, mut sel)) = ant_q.get_mut(*sel_e) {
                    // The queen cannot be ordered around except when wandering
                    if sel.kind == Ant::Queen && !player.has_trait(&Trait::WanderingQueen) {
                        continue;
                    }

                    // Skip commands onto himself
                    if ant_e != *sel_e {
                        if ant.health == 0. {
                            // If clicked on a corpse, go harvest it or protect the location
                            if sel.kind == Ant::Worker {
                                sel.command = Some(Behavior::HarvestCorpse(ant_e));
                                sel.action = Action::TargetedWalk(ant_e);
                            } else {
                                sel.command = Some(Behavior::ProtectLoc(loc));
                                sel.action = Action::Walk(loc);
                            }
                            success = true;
                        } else if ant.team != player.id {
                            // If clicked on an enemy, attack it
                            sel.command = Some(Behavior::Attack);
                            sel.action = Action::TargetedWalk(ant_e);
                            success = true;
                        } else {
                            // If clicked on an ally, protect it
                            sel.command = Some(Behavior::ProtectAnt(ant_e));
                            sel.action = Action::TargetedWalk(ant_e);
                            success = true;
                        }
                    }
                }
            }

            if !success {
                play_audio_ev.send(PlayAudioEv {
                    name: "error",
                    volume: 0.5,
                });
            }
        }
        _ => (),
    }
}

pub fn select_ants_from_rect(
    mut gizmos: Gizmos,
    ant_q: Query<(Entity, &Transform, &AntCmp)>,
    mut select_ants_ev: EventWriter<SelectAntEv>,
    players: Res<Players>,
    mut sbox: Local<SelectionBox>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let player = players.main();
    let (camera, global_t) = *camera;

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
                let min = Vec2::new(sbox.start.x.min(cursor.x), sbox.start.y.min(cursor.y));
                let max = Vec2::new(sbox.start.x.max(cursor.x), sbox.start.y.max(cursor.y));

                ant_q
                    .iter()
                    .filter(|(_, t, a)| {
                        a.team == player.id
                            && a.health > 0.
                            // Check if the ant is within the rectangle's bounds
                            && t.translation.x >= min.x
                            && t.translation.x <= max.x
                            && t.translation.y >= min.y
                            && t.translation.y <= max.y
                    })
                    .for_each(|(e, _, _)| {
                        select_ants_ev.send(SelectAntEv {
                            entity: e,
                            clean: false,
                        });
                    });

                *sbox = SelectionBox::default();
            }
        }
    }
}

pub fn select_ants_to_res(
    mut select_ant_ev: EventReader<SelectAntEv>,
    mut selection: ResMut<AntSelection>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_released(MouseButton::Left)
        && !keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        selection.0.clear();
    }

    for SelectAntEv { entity, clean } in select_ant_ev.read() {
        if !clean || !keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            selection.0.insert(*entity);
        } else {
            // Toggle selection state (insert if not present, remove if present)
            if !selection.0.remove(entity) {
                selection.0.insert(*entity);
            }
        }
    }
}

pub fn remove_command_from_selection(
    mut ant_q: Query<&mut AntCmp>,
    selection: Res<AntSelection>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Delete) {
        for sel_e in selection.0.iter() {
            if let Ok(mut ant) = ant_q.get_mut(*sel_e) {
                ant.command = None;
            }
        }
    }
}

pub fn update_selection_icons(
    mut commands: Commands,
    ant_q: Query<(Entity, &AntCmp)>,
    pin_q: Query<(Entity, &PinCmp)>,
    leaf_q: Query<(Entity, &GlobalTransform), With<Leaf>>,
    corpse_q: Query<(Entity, &GlobalTransform), With<Corpse>>,
    mut attack_q: Query<&mut Visibility, With<AttackCmp>>,
    mut defend_q: Query<&mut Visibility, (With<DefendCmp>, Without<AttackCmp>)>,
    children_q: Query<&Children>,
    mut pin_ev: EventWriter<PinEv>,
    map: Res<Map>,
    selection: Res<AntSelection>,
) {
    let mut to_pin: HashSet<Loc> = HashSet::new();
    let mut to_attack: HashSet<Entity> = HashSet::new();
    let mut to_defend: HashSet<Entity> = HashSet::new();

    for ant_e in &selection.0 {
        if let Ok((_, ant)) = ant_q.get(*ant_e) {
            match ant.command {
                Some(Behavior::Attack) => {
                    if let Action::TargetedWalk(entity) = ant.action {
                        to_attack.insert(entity);
                    }
                }
                Some(Behavior::Dig(loc)) => {
                    to_pin.insert(loc);
                }
                Some(Behavior::Harvest(entity)) => {
                    if let Ok((_, leaf_t)) = leaf_q.get(entity) {
                        to_pin.insert(map.get_loc(&leaf_t.translation()));
                    }
                }
                Some(Behavior::HarvestCorpse(entity)) | Some(Behavior::Heal(entity)) => {
                    if let Ok((_, corpse_t)) = corpse_q.get(entity) {
                        to_pin.insert(map.get_loc(&corpse_t.translation()));
                    }
                }
                Some(Behavior::ProtectAnt(entity)) => {
                    to_defend.insert(entity);
                }
                Some(Behavior::ProtectLoc(loc)) => {
                    to_pin.insert(loc);
                }
                _ => (),
            }
        }
    }

    for (pin_e, pin) in &pin_q {
        if to_pin.contains(&pin.loc) {
            to_pin.remove(&pin.loc);
        } else {
            commands.entity(pin_e).despawn();
        }
    }

    for loc in to_pin {
        pin_ev.send(PinEv(loc));
    }

    for (ant_e, _) in &ant_q {
        for child in children_q.iter_descendants(ant_e) {
            if let Ok(mut visibility) = defend_q.get_mut(child) {
                *visibility = if to_defend.contains(&ant_e) {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }

            if let Ok(mut visibility) = attack_q.get_mut(child) {
                *visibility = if to_attack.contains(&ant_e) {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}
