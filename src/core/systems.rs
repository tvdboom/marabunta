use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::{QueueAntEv, SpawnAntEv};
use crate::core::ants::selection::AntSelection;
use crate::core::audio::PlayAudioEv;
use crate::core::constants::{MAX_TRAITS, MONSTER_SPAWN_CHANCE};
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use crate::core::map::ui::utils::TextSize;
use crate::core::network::Population;
use crate::core::player::Players;
use crate::core::states::GameState;
use crate::core::utils::scale_duration;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::window::WindowResized;
use bevy_renet::renet::ClientId;
use rand::prelude::IteratorRandom;
use rand::{rng, Rng};
use std::f32::consts::PI;
use strum::IntoEnumIterator;

pub fn initialize_game(mut commands: Commands, mut game_settings: ResMut<GameSettings>) {
    commands.insert_resource(Players::default());
    commands.insert_resource(Map::default());
    commands.insert_resource(AntSelection::default());
    commands.insert_resource(Population::default());

    // Reset in-game settings
    game_settings.reset();
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
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_settings: ResMut<GameSettings>,
    players: Res<Players>,
    time: Res<Time>,
) {
    let player = players.get(0);

    let time = scale_duration(time.delta(), game_settings.speed);
    game_settings.trait_timer.tick(time);

    if game_settings.trait_timer.finished() && player.traits.len() < MAX_TRAITS {
        play_audio_ev.send(PlayAudioEv::new("message"));
        next_game_state.set(GameState::TraitSelection);
    }
}

pub fn check_keys(keyboard: Res<ButtonInput<KeyCode>>, mut players: ResMut<Players>) {
    let player = players.get_mut(0);

    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                player.resources += 1e4;
            }
        }
    }
}

pub fn spawn_enemies(
    mut queue_ant_ev: EventWriter<QueueAntEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut game_settings: ResMut<GameSettings>,
    mut players: ResMut<Players>,
    map: Res<Map>,
    time: Res<Time>,
) {
    let time = scale_duration(time.delta(), game_settings.speed);
    game_settings.enemy_timer.tick(time);

    if game_settings.enemy_timer.just_finished() {
        // NPCs spawn ants
        for player in players
            .0
            .iter_mut()
            .filter(|p| p.id != 0 && p.id != ClientId::MAX)
        {
            // Select ants that can be bought
            let ants = Ant::iter()
                .filter(|a| a.is_ant() && player.has_ant(a))
                .map(|a| AntCmp::new(&a, player))
                .filter(|a| player.resources >= a.price)
                .collect::<Vec<_>>();

            if !ants.is_empty() {
                // Compute saving probability
                let max_leaves = ants.iter().map(|a| a.price.leaves as u32).max().unwrap() as f32;
                let max_nutrients =
                    ants.iter().map(|a| a.price.nutrients as u32).max().unwrap() as f32;
                let save_prob = 0.6
                    + (max_leaves + max_nutrients)
                        / (max_leaves
                            + max_nutrients
                            + player.resources.leaves
                            + player.resources.nutrients);

                if rng().random::<f32>() >= save_prob {
                    let ant = ants.into_iter().choose(&mut rng()).unwrap();
                    player.resources -= ant.price;
                    queue_ant_ev.send(QueueAntEv {
                        id: player.id,
                        ant: ant.kind,
                    });
                }
            }
        }

        // Check holes to see if monsters should spawn
        map.tiles.iter().for_each(|tile| {
            if !tile.explored.is_empty() {
                if tile.texture_index == 64 && rng().random::<f32>() < MONSTER_SPAWN_CHANCE {
                    spawn_ant_ev.send(SpawnAntEv {
                        ant: AntCmp::base(&Ant::Wasp),
                        transform: Transform {
                            translation: Map::get_coord_from_xy(tile.x, tile.y).extend(0.),
                            rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                            ..default()
                        },
                    });
                } else if tile.texture_index == 65 && rng().random::<f32>() < MONSTER_SPAWN_CHANCE {
                    // Create random termite queue
                    let mut queue = vec![];
                    for _ in 1..=rng().random_range(2..=10) {
                        queue.push(match rng().random::<f32>() {
                            0.5..0.6 => Ant::BlackWingedTermite,
                            0.6..0.8 => Ant::BrownTermite,
                            0.8..0.9 => Ant::BrownWingedTermite,
                            0.9..0.97 => Ant::WhiteTermite,
                            0.97..1. => Ant::WhiteWingedTermite,
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
                        ant: AntCmp::base(&ant),
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
