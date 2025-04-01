use crate::core::assets::WorldAssets;
use crate::core::constants::{NORMAL_BUTTON_COLOR, PRESSED_BUTTON_COLOR};
use crate::core::game_settings::GameSettings;
use crate::core::menu::settings::SettingsBtn;
use crate::core::states::AudioState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

#[derive(Event)]
pub struct PlayAudioEv {
    pub name: &'static str,
    pub volume: f64,
}

impl PlayAudioEv {
    pub fn new(name: &'static str) -> Self {
        Self { name, volume: 1. }
    }
}

#[derive(Component)]
pub struct MusicBtnCmp;

#[derive(Event)]
pub struct ChangeAudioEv(pub Option<AudioState>);

pub fn setup_music_btn(mut commands: Commands, assets: Local<WorldAssets>) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(5.),
            height: Val::Percent(5.),
            right: Val::Percent(0.),
            top: Val::Percent(3.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((ImageNode::new(assets.image("sound")), MusicBtnCmp))
                .observe(|_: Trigger<Pointer<Click>>, mut commands: Commands| {
                    commands.queue(|w: &mut World| {
                        w.send_event(ChangeAudioEv(None));
                    })
                });
        });
}

pub fn play_music(assets: Local<WorldAssets>, audio: Res<Audio>) {
    audio
        .play(assets.audio("music"))
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.03)
        .looped();
}

pub fn change_audio_event(
    mut change_audio_ev: EventReader<ChangeAudioEv>,
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    mut settings_btn: Query<(&mut BackgroundColor, &SettingsBtn)>,
    mut game_settings: ResMut<GameSettings>,
    audio_state: Res<State<AudioState>>,
    mut next_audio_state: ResMut<NextState<AudioState>>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    for ev in change_audio_ev.read() {
        game_settings.audio = ev.0.unwrap_or(match *audio_state.get() {
            AudioState::Mute => AudioState::NoMusic,
            AudioState::NoMusic => AudioState::Sound,
            AudioState::Sound => AudioState::Mute,
        });

        if let Ok(mut node) = btn_q.get_single_mut() {
            node.image = match game_settings.audio {
                AudioState::Mute => {
                    next_audio_state.set(AudioState::Mute);
                    assets.image("mute")
                }
                AudioState::NoMusic => {
                    audio.stop();
                    next_audio_state.set(AudioState::NoMusic);
                    assets.image("no-music")
                }
                AudioState::Sound => {
                    next_audio_state.set(AudioState::Sound);
                    assets.image("sound")
                }
            };
        }

        for (mut bgcolor, setting) in &mut settings_btn {
            if matches!(
                setting,
                SettingsBtn::Mute | SettingsBtn::NoMusic | SettingsBtn::Sound
            ) {
                bgcolor.0 = if (*setting == SettingsBtn::Mute
                    && game_settings.audio == AudioState::Mute)
                    || (*setting == SettingsBtn::NoMusic
                        && game_settings.audio == AudioState::NoMusic)
                    || (*setting == SettingsBtn::Sound && game_settings.audio == AudioState::Sound)
                {
                    PRESSED_BUTTON_COLOR
                } else {
                    NORMAL_BUTTON_COLOR
                };
            }
        }
    }
}

pub fn toggle_music_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut change_audio_ev: EventWriter<ChangeAudioEv>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        change_audio_ev.send(ChangeAudioEv(None));
    }
}

pub fn play_audio_event(
    mut ev: EventReader<PlayAudioEv>,
    audio_state: Res<State<AudioState>>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    if *audio_state.get() != AudioState::Mute {
        for PlayAudioEv { name, volume } in ev.read() {
            audio.play(assets.audio(name)).with_volume(*volume);
        }
    }
}
