use crate::core::menu::constants::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::core::menu::utils::{create_button_node, create_button_text, create_root_node};
use crate::utils::NameFromEnum;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainMenuComponent;

#[derive(Component, Debug)]
pub enum MainMenuBtn {
    Multiplayer,
    Quit,
}

pub fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((create_root_node(), MainMenuComponent))
        .with_children(|parent| {
            parent
                .spawn((create_button_node(), Button, MainMenuBtn::Multiplayer))
                .with_children(|parent| {
                    parent.spawn(create_button_text(
                        MainMenuBtn::Multiplayer.as_string(),
                        &asset_server,
                    ));
                });

            parent
                .spawn((create_button_node(), Button, MainMenuBtn::Quit))
                .with_children(|parent| {
                    parent.spawn(create_button_text(
                        MainMenuBtn::Quit.as_string(),
                        &asset_server,
                    ));
                });
        });
}

pub fn btn_interact(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MainMenuBtn>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match *interaction {
            Interaction::None => NORMAL_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::Pressed => PRESSED_BUTTON.into(),
        }
    }
}

// pub fn btn_listeners(
//     mut exit: EventWriter<AppExit>,
//     mut commands: Commands,
//     mut state: ResMut<State<AppState>>,
//     mut interaction_query: Query<(&Interaction, &MainMenuBtn), Changed<Interaction>>,
// ) {
//     for (interaction, btn) in interaction_query.iter_mut() {
//         if let Interaction::Clicked = *interaction {
//             match btn {
//                 MainMenuBtn::Multiplayer => {
//                     state
//                         .set(AppState::MenuOnline)
//                         .expect("Could not change state.");
//                 }
//                 MainMenuBtn::LocalMatch => {
//                     create_synctest_session(&mut commands);
//                     state
//                         .set(AppState::RoundLocal)
//                         .expect("Could not change state.");
//                 }
//                 MainMenuBtn::Quit => {
//                     exit.send(AppExit);
//                 }
//             }
//         }
//     }
// }
