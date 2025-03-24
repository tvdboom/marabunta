use crate::core::game_settings::GameSettings;
use crate::core::player::Players;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::{Read, Write};

use crate::core::ants::components::AntCmp;
use crate::core::ants::events::SpawnAntEv;
use crate::core::map::map::Map;
use crate::core::states::{AppState, AudioState};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

pub type PopulationT = HashMap<Entity, (Transform, AntCmp)>;

#[derive(Serialize, Deserialize)]
pub struct SaveAll {
    pub game_settings: GameSettings,
    pub players: Players,
    pub map: Map,
    pub population: PopulationT,
}

#[derive(Event)]
pub struct LoadGameEv;

#[derive(Event)]
pub struct SaveGameEv;

fn save_to_bin(file_path: &str, data: &SaveAll) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(&bincode::serialize(data).expect("Failed to serialize data."))?;
    Ok(())
}

fn load_from_bin(file_path: &str) -> io::Result<SaveAll> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let data: SaveAll = bincode::deserialize(&buffer).expect("Failed to deserialize data.");
    Ok(data)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game(
    mut commands: Commands,
    mut load_game_ev: EventReader<LoadGameEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_audio_state: ResMut<NextState<AudioState>>,
) {
    for _ in load_game_ev.read() {
        if let Some(file_path) = FileDialog::new().pick_file() {
            let file_path_str = file_path.to_string_lossy().to_string();
            let data = load_from_bin(&file_path_str).expect("Failed to load the game.");

            next_audio_state.set(data.game_settings.audio);
            commands.insert_resource(data.game_settings);

            commands.insert_resource(data.players);
            commands.insert_resource(data.map);

            for (_, (transform, ant)) in data.population {
                spawn_ant_ev.send(SpawnAntEv { ant, transform });
            }

            next_app_state.set(AppState::Game);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(
    mut save_game_ev: EventReader<SaveGameEv>,
    game_settings: Res<GameSettings>,
    players: Res<Players>,
    map: Res<Map>,
    ant_q: Query<(Entity, &Transform, &AntCmp)>,
) {
    for _ in save_game_ev.read() {
        if let Some(mut file_path) = FileDialog::new().save_file() {
            if !file_path.extension().map(|e| e == "bin").unwrap_or(false) {
                file_path.set_extension("bin");
            }

            let file_path_str = file_path.to_string_lossy().to_string();
            let data = SaveAll {
                game_settings: game_settings.clone(),
                players: players.clone(),
                map: map.clone(),
                population: ant_q
                    .iter()
                    .map(|(e, t, a)| (e, (t.clone(), a.clone())))
                    .collect(),
            };

            save_to_bin(&file_path_str, &data).expect("Failed to save the game.");
        }
    }
}
