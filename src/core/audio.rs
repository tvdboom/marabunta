use crate::core::assets::WorldAssets;
use crate::core::states::MusicState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct BackgroundMusicBtn;

pub fn play_music(assets: Local<WorldAssets>, audio: Res<Audio>) {
    audio
        .play(assets.audio("music"))
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.05)
        .looped();
}

pub fn stop_music(audio: Res<Audio>) {
    audio.stop();
}

pub fn setup_music_btn(
    mut commands: Commands,
    btn_q: Query<Entity, With<BackgroundMusicBtn>>,
    music_state: Res<State<MusicState>>,
    assets: Local<WorldAssets>,
) {
    for btn_e in btn_q.iter() {
        commands.entity(btn_e).despawn_recursive();
    }

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
            parent.spawn((
                ImageNode::new(assets.image(if *music_state.get() == MusicState::Playing {
                    "sound"
                } else {
                    "mute"
                })),
                Button,
                BackgroundMusicBtn,
            ));
        });
}

pub fn music_btn_listener(
    interaction_q: Query<
        &Interaction,
        (With<Button>, With<BackgroundMusicBtn>, Changed<Interaction>),
    >,
    music_state: Res<State<MusicState>>,
    mut next_music_state: ResMut<NextState<MusicState>>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            match music_state.get() {
                MusicState::Playing => next_music_state.set(MusicState::Stopped),
                MusicState::Stopped => next_music_state.set(MusicState::Playing),
            }
        }
    }
}
