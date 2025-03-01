use crate::core::ants::components::Ant;
use crate::core::map::map::Map;
use crate::core::player::Player;
use crate::core::resources::{GameSettings, Population};
use bevy::prelude::*;

pub fn initialize_game(mut commands: Commands) {
    commands.insert_resource(GameSettings::default());
    commands.insert_resource(Player::default());
    commands.insert_resource(Map::default());
    commands.insert_resource(Population::default());
}

pub fn check_keys(keyboard: Res<ButtonInput<KeyCode>>, mut player: ResMut<Player>) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        player.queue.push(Ant::BlackAnt);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        player.queue.push(Ant::BlackBullet);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        player.queue.push(Ant::BlackSoldier);
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        player.queue.push(Ant::GoldTail);
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        player.queue.push(Ant::TrapJaw);
    }
}
