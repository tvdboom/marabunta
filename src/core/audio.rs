use crate::core::assets::WorldAssets;
use crate::core::game_settings::GameSettings;
use crate::core::states::AudioState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct MusicBtnCmp;

#[derive(Event)]
pub struct ToggleMusicEv;

pub fn play_music(
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    mut game_settings: ResMut<GameSettings>,
    assets: Local<WorldAssets>,
    audio: Res<Audio>,
) {
    audio
        .play(assets.audio("music"))
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.03)
        .looped();

    game_settings.audio = AudioState::Playing;
    if let Ok(mut node) = btn_q.get_single_mut() {
        node.image = assets.image("sound");
    }
}

pub fn stop_music(
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    mut game_settings: ResMut<GameSettings>,
    assets: Local<WorldAssets>,
    audio: Res<Audio>,
) {
    audio.stop();

    game_settings.audio = AudioState::Stopped;
    if let Ok(mut node) = btn_q.get_single_mut() {
        node.image = assets.image("mute");
    }
}

pub fn setup_music_btn(mut commands: Commands, assets: Local<WorldAssets>) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Px(40.),
            height: Val::Px(40.),
            right: Val::Px(20.),
            top: Val::Px(20.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((ImageNode::new(assets.image("sound")), MusicBtnCmp))
                .observe(|_click: Trigger<Pointer<Click>>, mut commands: Commands| {
                    commands.queue(|w: &mut World| {
                        w.send_event(ToggleMusicEv);
                    })
                });
        });
}

pub fn toggle_music(
    mut toggle_music_ev: EventReader<ToggleMusicEv>,
    audio_state: Res<State<AudioState>>,
    mut next_audio_state: ResMut<NextState<AudioState>>,
) {
    for _ in toggle_music_ev.read() {
        match *audio_state.get() {
            AudioState::Playing => next_audio_state.set(AudioState::Stopped),
            AudioState::Stopped => next_audio_state.set(AudioState::Playing),
        }
    }
}

pub fn toggle_music_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut toggle_music_ev: EventWriter<ToggleMusicEv>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        toggle_music_ev.send(ToggleMusicEv);
    }
}
