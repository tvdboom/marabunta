use crate::core::ants::components::{Ant, AntCmp};
use crate::core::map::map::Map;
use crate::core::player::Player;
use crate::core::resources::{GameSettings, Population};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;

pub fn initialize_game(mut commands: Commands) {
    commands.insert_resource(GameSettings::default());
    commands.insert_resource(Player::default());
    commands.insert_resource(Map::default());
    commands.insert_resource(Population::default());
}

pub fn check_keys(keyboard: Res<ButtonInput<KeyCode>>, mut player: ResMut<Player>) {
    let mapping: HashMap<KeyCode, Ant> = HashMap::from([
        (KeyCode::Digit1, Ant::BlackAnt),
        (KeyCode::Digit2, Ant::BlackBullet),
        (KeyCode::Digit3, Ant::BlackSoldier),
        (KeyCode::Digit4, Ant::GoldTail),
        (KeyCode::Digit5, Ant::TrapJaw),
    ]);

    for (key, ant) in mapping.iter() {
        let ant = AntCmp::new(ant, player.id);
        if keyboard.just_pressed(*key) && player.food >= ant.price {
            player.food -= ant.price;
            player.queue.push(ant.kind);
        }
    }

    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                player.food += 1e5;
            }
        }
    }
}
