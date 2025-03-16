use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::{QueueAntEv, SpawnAntEv};
use crate::core::assets::WorldAssets;
use crate::core::constants::MAX_TRAITS;
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use crate::core::map::selection::AntSelection;
use crate::core::map::ui::utils::TextSize;
use crate::core::network::Population;
use crate::core::player::Player;
use crate::core::states::GameState;
use crate::core::utils::scale_duration;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::window::WindowResized;
use bevy_kira_audio::{Audio, AudioControl};
use rand::{rng, Rng};
use std::f32::consts::PI;
use strum::IntoEnumIterator;

pub fn initialize_game(mut commands: Commands) {
    commands.insert_resource(GameSettings::default());
    commands.insert_resource(Player::default());
    commands.insert_resource(Map::default());
    commands.insert_resource(Population::default());
    commands.insert_resource(AntSelection::default());
}

pub fn on_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut text: Query<(&mut TextFont, &TextSize)>,
) {
    for ev in resize_reader.read() {
        for (mut text, size) in text.iter_mut() {
            text.font_size = size.0 * ev.height / 460.
        }
    }
}

pub fn check_trait_timer(
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_settings: ResMut<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    // Only the host switches states -> the clients follow after the update
    if player.id == 0 {
        let time = scale_duration(time.delta(), game_settings.speed);
        game_settings.trait_timer.tick(time);

        if game_settings.trait_timer.finished() && player.traits.len() < MAX_TRAITS {
            audio.play(assets.audio("message"));
            next_game_state.set(GameState::TraitSelection);
        }
    }
}

pub fn check_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: ResMut<Player>,
    mut queue_ant_ev: EventWriter<QueueAntEv>,
) {
    for ant in Ant::iter().filter(|a| player.has_ant(a)) {
        if matches!(AntCmp::base(&ant).key, Some(key) if keyboard.just_pressed(key)) {
            queue_ant_ev.send(QueueAntEv { ant });
        }
    }

    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                player.food += 1e4;
            }
        }
    }
}

pub fn spawn_enemies(
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut game_settings: ResMut<GameSettings>,
    player: Res<Player>,
    map: Res<Map>,
    time: Res<Time>,
) {
    // Enemies only spawn from the host player's pc
    if player.id == 0 {
        let time = scale_duration(time.delta(), game_settings.speed);
        game_settings.enemy_timer.tick(time);

        if game_settings.enemy_timer.just_finished() {
            map.tiles.iter().for_each(|tile| {
                if tile.visible.contains(&player.id) {
                    if tile.texture_index == 64 && rng().random::<f32>() < 0.001 {
                        spawn_ant_ev.send(SpawnAntEv {
                            ant: AntCmp::new(&Ant::Wasp, &player),
                            transform: Transform {
                                translation: Map::get_coord_from_xy(tile.x, tile.y).extend(0.),
                                rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                                ..default()
                            },
                        });
                    } else if tile.texture_index == 65 && rng().random::<f32>() < 0.01 {
                        // Create random termite queue
                        let mut queue = vec![];
                        for _ in 1..=rng().random_range(2..=10) {
                            queue.push(match rng().random::<f32>() {
                                0.6..0.7 => Ant::BlackWingedTermite,
                                0.7..0.85 => Ant::BrownTermite,
                                0.85..0.9 => Ant::BrownWingedTermite,
                                0.95..0.99 => Ant::WhiteTermite,
                                0.99..1. => Ant::WhiteWingedTermite,
                                _ => Ant::BlackTermite,
                            });
                        }

                        game_settings.termite_queue = HashMap::from([((tile.x, tile.y), queue)]);
                    }
                }
            });

            // Spawn termites gradually from the termite queue
            for ((x, y), queue) in game_settings.termite_queue.iter_mut() {
                if rng().random::<f32>() < 0.2 {
                    if let Some(ant) = queue.pop() {
                        spawn_ant_ev.send(SpawnAntEv {
                            ant: AntCmp::new(&ant, &player),
                            transform: Transform {
                                translation: Map::get_coord_from_xy(*x, *y).extend(0.),
                                rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                                ..default()
                            },
                        });
                    }
                }
            }
        }
    }
}
