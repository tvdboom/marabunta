use crate::core::assets::WorldAssets;
use crate::core::constants::MAX_Z_SCORE;
use crate::core::resources::GameSettings;
use crate::core::states::PauseState;
use bevy::color::palettes::basic::WHITE;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};

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

pub fn pause_game(mut vis_q: Query<&mut Visibility, With<PauseCmp>>, audio: Res<Audio>) {
    *vis_q.single_mut() = Visibility::Visible;
    audio.pause();
}

pub fn unpause_game(
    mut vis_q: Query<&mut Visibility, With<PauseCmp>>,
    mut game_settings: ResMut<GameSettings>,
    audio: Res<Audio>,
) {
    // PauseWrapper not yet spawned at first iteration
    if let Ok(mut e) = vis_q.get_single_mut() {
        if game_settings.speed == 0. {
            game_settings.speed = 1.;
        }
        *e = Visibility::Hidden;
        audio.resume();
    }
}

pub fn toggle_pause_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match pause_state.get() {
            PauseState::Running => next_pause_state.set(PauseState::Paused),
            PauseState::Paused => next_pause_state.set(PauseState::Running),
        }
    }
}
