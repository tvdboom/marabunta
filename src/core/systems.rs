use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::QueueAntEv;
use crate::core::assets::WorldAssets;
use crate::core::constants::MAX_TRAITS;
use crate::core::map::map::Map;
use crate::core::player::Player;
use crate::core::resources::{GameSettings, Population};
use crate::core::states::GameState;
use crate::core::utils::scale_duration;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};
use strum::IntoEnumIterator;

pub fn initialize_game(mut commands: Commands) {
    commands.insert_resource(GameSettings::default());
    commands.insert_resource(Player::default());
    commands.insert_resource(Map::default());
    commands.insert_resource(Population::default());
}

pub fn check_trait_timer(
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_settings: ResMut<GameSettings>,
    player: Res<Player>,
    time: Res<Time>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    let time = scale_duration(time.delta(), game_settings.speed);
    game_settings.trait_timer.tick(time);

    if game_settings.trait_timer.finished() && player.traits.len() < MAX_TRAITS {
        audio.play(assets.audio("message"));
        next_game_state.set(GameState::TraitSelection);
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
