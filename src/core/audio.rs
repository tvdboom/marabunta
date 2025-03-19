use crate::core::assets::WorldAssets;
use crate::core::game_settings::GameSettings;
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
pub struct ToggleAudioEv;

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
                .observe(|_click: Trigger<Pointer<Click>>, mut commands: Commands| {
                    commands.queue(|w: &mut World| {
                        w.send_event(ToggleAudioEv);
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

pub fn toggle_music_event(
    mut toggle_music_ev: EventReader<ToggleAudioEv>,
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    mut game_settings: ResMut<GameSettings>,
    audio_state: Res<State<AudioState>>,
    mut next_audio_state: ResMut<NextState<AudioState>>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    for _ in toggle_music_ev.read() {
        let image = match *audio_state.get() {
            AudioState::Sound => {
                audio.stop();

                game_settings.audio = AudioState::NoMusic;
                next_audio_state.set(AudioState::NoMusic);
                assets.image("no-music")
            }
            AudioState::NoMusic => {
                game_settings.audio = AudioState::Mute;
                next_audio_state.set(AudioState::Mute);
                assets.image("mute")
            }
            AudioState::Mute => {
                game_settings.audio = AudioState::Sound;
                next_audio_state.set(AudioState::Sound);
                assets.image("sound")
            }
        };

        if let Ok(mut node) = btn_q.get_single_mut() {
            node.image = image;
        }
    }
}

pub fn toggle_music_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut toggle_music_ev: EventWriter<ToggleAudioEv>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        toggle_music_ev.send(ToggleAudioEv);
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
