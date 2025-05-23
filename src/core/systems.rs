use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::QueueAntEv;
use crate::core::ants::selection::{AntSelection, GroupSelection};
use crate::core::audio::PlayAudioEv;
use crate::core::constants::MAX_TRAITS;
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use crate::core::map::ui::utils::TextSize;
use crate::core::multiplayer::EntityMap;
use crate::core::player::Players;
use crate::core::states::GameState;
use crate::core::traits::AfterTraitCount;
use crate::core::utils::scale_duration;
use bevy::prelude::*;
use bevy::window::WindowResized;
use bevy_renet::renet::ClientId;
use rand::prelude::IteratorRandom;
use rand::{rng, Rng};
use strum::IntoEnumIterator;

pub fn initialize_game(mut commands: Commands, mut game_settings: ResMut<GameSettings>) {
    commands.insert_resource(Players::default());
    commands.insert_resource(Map::default());
    commands.insert_resource(AntSelection::default());
    commands.insert_resource(GroupSelection::default());
    commands.insert_resource(EntityMap::default());
    commands.insert_resource(AfterTraitCount::default());

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
    let player = players.main();

    // Only the host starts the trait selection
    if player.id == 0 {
        let time = scale_duration(time.delta(), game_settings.speed);
        game_settings.trait_timer.tick(time);

        if game_settings.trait_timer.finished() && player.traits.len() < MAX_TRAITS {
            play_audio_ev.send(PlayAudioEv::new("message"));
            next_game_state.set(GameState::TraitSelection);
        }
    }
}

pub fn check_keys(keyboard: Res<ButtonInput<KeyCode>>, mut players: ResMut<Players>) {
    let player = players.main_mut();

    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                player.resources += 1e4;
            }
        }
    }
}

pub fn npc_buy_ants(mut queue_ant_ev: EventWriter<QueueAntEv>, mut players: ResMut<Players>) {
    for player in players
        .0
        .iter_mut()
        .filter(|p| !p.is_human() && p.id != ClientId::MAX)
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
            let max_nutrients = ants.iter().map(|a| a.price.nutrients as u32).max().unwrap() as f32;
            let save_prob = 0.55
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
}
