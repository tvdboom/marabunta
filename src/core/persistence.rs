use crate::core::game_settings::GameSettings;
use crate::core::player::Player;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::{fs, io};
use uuid::Uuid;

use crate::core::ants::components::AntCmp;
use crate::core::ants::events::SpawnAntEv;
use crate::core::map::map::Map;
use crate::core::states::{AppState, AudioState, GameState};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

pub type PopulationT = HashMap<Uuid, (Transform, AntCmp)>;

#[derive(Serialize, Deserialize)]
pub struct SaveAll {
    pub game_settings: GameSettings,
    pub player: Player,
    pub map: Map,
    pub population: PopulationT,
}

#[derive(Event)]
pub struct LoadGameEv;

#[derive(Event)]
pub struct SaveGameEv;

fn save_to_json(file_path: &str, data: &SaveAll) -> io::Result<()> {
    let json_data = serde_json::to_string_pretty(data)?;

    let mut file = File::create(file_path)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

fn load_from_json(file_path: &str) -> io::Result<SaveAll> {
    let json_data = fs::read_to_string(file_path)?;
    let data: SaveAll = serde_json::from_str(&json_data)?;
    Ok(data)
}

/// Load a game from a JSON file
#[cfg(not(target_arch = "wasm32"))]
pub fn load_game(
    mut commands: Commands,
    mut load_game_ev: EventReader<LoadGameEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_audio_state: ResMut<NextState<AudioState>>,
) {
    for _ in load_game_ev.read() {
        if let Some(file_path) = FileDialog::new().pick_file() {
            let file_path_str = file_path.to_string_lossy().to_string();
            let data = load_from_json(&file_path_str).expect("Failed to load the game.");

            next_audio_state.set(data.game_settings.audio);
            commands.insert_resource(data.game_settings);

            commands.insert_resource(data.player);
            commands.insert_resource(data.map);

            for (_, (transform, ant)) in data.population {
                spawn_ant_ev.send(SpawnAntEv { ant, transform });
            }

            next_app_state.set(AppState::Game);
            next_game_state.set(GameState::Paused);
        }
    }
}

/// Save the game to a JSON file
#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(
    mut save_game_ev: EventReader<SaveGameEv>,
    game_settings: Res<GameSettings>,
    player: &Player,
    map: Res<Map>,
    ant_q: Query<(&Transform, &AntCmp)>,
) {
    for _ in save_game_ev.read() {
        if let Some(mut file_path) = FileDialog::new().save_file() {
            if !file_path.extension().map(|e| e == "json").unwrap_or(false) {
                file_path.set_extension("json");
            }

            let file_path_str = file_path.to_string_lossy().to_string();
            let data = SaveAll {
                game_settings: game_settings.clone(),
                player: player.clone(),
                map: map.clone(),
                population: ant_q
                    .iter()
                    .map(|(t, a)| (a.id, (t.clone(), a.clone())))
                    .collect(),
            };

            save_to_json(&file_path_str, &data).expect("Failed to save the game.");
        }
    }
}
