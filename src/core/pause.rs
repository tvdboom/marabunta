use crate::core::assets::WorldAssets;
use crate::core::constants::{GAME_SPEED_STEP, MAX_GAME_SPEED, MAX_Z_SCORE};
use crate::core::resources::{GameMode, GameSettings};
use crate::core::states::GameState;
use bevy::color::palettes::basic::WHITE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;

#[derive(Component)]
pub struct PauseCmp;

pub fn spawn_pause_banner(mut commands: Commands, assets: Local<WorldAssets>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            PickingBehavior::IGNORE,
            PauseCmp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::from("Paused"),
                TextColor(Color::from(WHITE)),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font: assets.font("FiraSans-Bold"),
                    font_size: 55.,
                    ..default()
                },
                Transform::from_xyz(0., 0., MAX_Z_SCORE),
            ));
        });
}

pub fn pause_game(mut vis_q: Query<&mut Visibility, With<PauseCmp>>) {
    *vis_q.single_mut() = Visibility::Visible;
}

pub fn unpause_game(
    mut vis_q: Query<&mut Visibility, With<PauseCmp>>,
    mut game_settings: ResMut<GameSettings>,
) {
    // PauseWrapper not yet spawned at first iteration
    if let Ok(mut e) = vis_q.get_single_mut() {
        if game_settings.speed == 0. {
            game_settings.speed = 1.;
        }
        *e = Visibility::Hidden;
    }
}

pub fn toggle_pause_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    server: Option<Res<RenetServer>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_settings: ResMut<GameSettings>,
) {
    // Only the host can pause the game in multiplayer mode
    if game_settings.mode == GameMode::SinglePlayer || server.is_some() {
        if keyboard.just_pressed(KeyCode::Space) {
            match game_state.get() {
                GameState::Running => next_game_state.set(GameState::Paused),
                GameState::Paused => next_game_state.set(GameState::Running),
                _ => (),
            }
        }

        if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            if keyboard.just_pressed(KeyCode::ArrowLeft) && game_settings.speed >= GAME_SPEED_STEP {
                game_settings.speed -= GAME_SPEED_STEP;
                if game_settings.speed == 0. {
                    next_game_state.set(GameState::Paused);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) && game_settings.speed <= MAX_GAME_SPEED {
                game_settings.speed += GAME_SPEED_STEP;
                if game_settings.speed == GAME_SPEED_STEP {
                    next_game_state.set(GameState::Running);
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        match game_state.get() {
            GameState::Running => next_game_state.set(GameState::InGameMenu),
            GameState::Paused => next_game_state.set(GameState::InGameMenu),
            GameState::InGameMenu => next_game_state.set(GameState::Running),
        }
    }
}
