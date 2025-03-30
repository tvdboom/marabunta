use crate::core::assets::WorldAssets;
use crate::core::audio::ChangeAudioEv;
use crate::core::constants::*;
use crate::core::game_settings::GameSettings;
use crate::core::map::ui::utils::add_text;
use crate::core::states::AudioState;
use crate::utils::NameFromEnum;
use bevy::hierarchy::{ChildBuild, ChildBuilder};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum_macros::EnumIter;

#[derive(Component, Clone, Debug)]
pub enum SettingsBtn {
    Black,
    Red,
    None,
    Half,
    Full,
    Zero,
    One,
    Two,
    Three,
    Mute,
    NoMusic,
    Sound,
}

#[derive(EnumIter, Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum AntColor {
    #[default]
    Black,
    Red,
}

impl AntColor {
    pub fn inverse(&self) -> Self {
        match self {
            AntColor::Black => AntColor::Red,
            AntColor::Red => AntColor::Black,
        }
    }
}

#[derive(EnumIter, Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum FogOfWar {
    None,
    Half,
    #[default]
    Full,
}

fn match_setting(setting: &SettingsBtn, game_settings: &GameSettings) -> bool {
    match setting {
        SettingsBtn::Black => game_settings.color == AntColor::Black,
        SettingsBtn::Red => game_settings.color == AntColor::Red,
        SettingsBtn::None => game_settings.fog_of_war == FogOfWar::None,
        SettingsBtn::Half => game_settings.fog_of_war == FogOfWar::Half,
        SettingsBtn::Full => game_settings.fog_of_war == FogOfWar::Full,
        SettingsBtn::Zero => game_settings.n_opponents == 0,
        SettingsBtn::One => game_settings.n_opponents == 1,
        SettingsBtn::Two => game_settings.n_opponents == 2,
        SettingsBtn::Three => game_settings.n_opponents == 3,
        SettingsBtn::Mute => game_settings.audio == AudioState::Mute,
        SettingsBtn::NoMusic => game_settings.audio == AudioState::NoMusic,
        SettingsBtn::Sound => game_settings.audio == AudioState::Sound,
    }
}

pub fn recolor_label<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(Trigger<E>, Query<(&mut BackgroundColor, &SettingsBtn)>, ResMut<GameSettings>) {
    move |ev, mut bgcolor_q, game_settings| {
        if let Ok((mut bgcolor, setting)) = bgcolor_q.get_mut(ev.entity()) {
            // Don't change the color of selected buttons
            if !match_setting(&setting, &game_settings) {
                bgcolor.0 = color;
            }
        };
    }
}

pub fn on_click_label_button(
    trigger: Trigger<Pointer<Click>>,
    mut btn_q: Query<(&mut BackgroundColor, &SettingsBtn)>,
    mut game_settings: ResMut<GameSettings>,
    mut change_audio_ev: EventWriter<ChangeAudioEv>,
) {
    match btn_q.get(trigger.entity()).unwrap().1 {
        SettingsBtn::Black => game_settings.color = AntColor::Black,
        SettingsBtn::Red => game_settings.color = AntColor::Red,
        SettingsBtn::None => game_settings.fog_of_war = FogOfWar::None,
        SettingsBtn::Half => game_settings.fog_of_war = FogOfWar::Half,
        SettingsBtn::Full => game_settings.fog_of_war = FogOfWar::Full,
        SettingsBtn::Zero => game_settings.n_opponents = 0,
        SettingsBtn::One => game_settings.n_opponents = 1,
        SettingsBtn::Two => game_settings.n_opponents = 2,
        SettingsBtn::Three => game_settings.n_opponents = 3,
        SettingsBtn::Mute => {
            game_settings.audio = AudioState::Mute;
            change_audio_ev.send(ChangeAudioEv(Some(AudioState::Mute)));
        }
        SettingsBtn::NoMusic => {
            game_settings.audio = AudioState::NoMusic;
            change_audio_ev.send(ChangeAudioEv(Some(AudioState::NoMusic)));
        }
        SettingsBtn::Sound => {
            game_settings.audio = AudioState::Sound;
            change_audio_ev.send(ChangeAudioEv(Some(AudioState::Sound)));
        }
    }

    // Reset the color of the other buttons
    for (mut bgcolor, setting) in &mut btn_q {
        if !match_setting(setting, &game_settings) {
            bgcolor.0 = NORMAL_BUTTON_COLOR.into();
        }
    }
}

pub fn spawn_label(
    parent: &mut ChildBuilder,
    title: &str,
    buttons: Vec<SettingsBtn>,
    game_settings: &GameSettings,
    assets: &WorldAssets,
    window: &Window,
) {
    parent.spawn(add_text(
        title,
        "bold",
        SUBTITLE_TEXT_SIZE,
        &assets,
        &window,
    ));

    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            padding: UiRect {
                top: Val::Percent(1.),
                left: Val::Percent(5.),
                right: Val::Percent(5.),
                bottom: Val::Percent(5.),
            },
            ..default()
        })
        .with_children(|parent| {
            for item in buttons.iter() {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(30.),
                            height: Val::Percent(100.),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::all(Val::Percent(1.)),
                            ..default()
                        },
                        BackgroundColor(if match_setting(item, game_settings) {
                            PRESSED_BUTTON_COLOR.into()
                        } else {
                            NORMAL_BUTTON_COLOR.into()
                        }),
                        item.clone(),
                        Button,
                    ))
                    .observe(recolor_label::<Pointer<Over>>(HOVERED_BUTTON_COLOR))
                    .observe(recolor_label::<Pointer<Out>>(NORMAL_BUTTON_COLOR))
                    .observe(recolor_label::<Pointer<Down>>(PRESSED_BUTTON_COLOR))
                    .observe(recolor_label::<Pointer<Up>>(HOVERED_BUTTON_COLOR))
                    .observe(on_click_label_button)
                    .with_children(|parent| {
                        parent.spawn(add_text(
                            item.to_title(),
                            "bold",
                            LABEL_TEXT_SIZE,
                            assets,
                            window,
                        ));
                    });
            }
        });
}
