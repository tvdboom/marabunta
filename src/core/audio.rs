use crate::core::assets::WorldAssets;
use crate::core::states::MusicState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct MusicBtnCmp;

#[derive(Event)]
pub struct ToggleMusicEv;

pub fn play_music(
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    assets: Local<WorldAssets>,
    audio: Res<Audio>,
) {
    audio
        .play(assets.audio("music"))
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.05)
        .looped();

    if let Ok(mut node) = btn_q.get_single_mut() {
        node.image = assets.image("sound");
    }
}

pub fn stop_music(
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    assets: Local<WorldAssets>,
    audio: Res<Audio>,
) {
    audio.stop();

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
    music_state: Res<State<MusicState>>,
    mut next_music_state: ResMut<NextState<MusicState>>,
) {
    for _ in toggle_music_ev.read() {
        match *music_state.get() {
            MusicState::Playing => next_music_state.set(MusicState::Stopped),
            MusicState::Stopped => next_music_state.set(MusicState::Playing),
        }
    }
}
