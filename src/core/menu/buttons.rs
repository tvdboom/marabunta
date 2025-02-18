use crate::core::assets::WorldAssets;
use crate::core::menu::constants::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::core::menu::utils::{add_text, recolor};
use crate::core::network::{new_renet_client, new_renet_server, ServerMessage};
use crate::core::player::Player;
use crate::core::states::GameState;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy_renet::netcode::{NetcodeClientTransport, NetcodeServerTransport};
use bevy_renet::renet::{DefaultChannel, RenetClient, RenetServer};

#[derive(Component)]
pub struct MenuCmp;

#[derive(Component, Clone, Debug)]
pub enum MenuBtn {
    HostGame,
    FindGame,
    Play,
    BackToMenu,
    Quit,
}

#[derive(Component)]
pub struct LobbyTextCmp;

pub fn on_click_menu_button(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    btn_q: Query<&MenuBtn>,
    mut next_game_state: ResMut<NextState<GameState>>,
    server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    match btn_q.get(click.entity()).unwrap() {
        MenuBtn::HostGame => {
            // Remove client resources if they exist
            if client.is_some() {
                commands.remove_resource::<RenetClient>();
                commands.remove_resource::<NetcodeClientTransport>();
            }

            let (server, transport) = new_renet_server();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_game_state.set(GameState::Lobby);
        }
        MenuBtn::FindGame => {
            let (server, transport) = new_renet_client();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_game_state.set(GameState::Lobby);
        }
        MenuBtn::Play => {
            let mut server = server.unwrap();

            // Send the start game signal to all clients with their player number
            for (i, client) in server.clients_id().iter().enumerate() {
                let message = bincode::serialize(&ServerMessage::StartGame(i + 1)).unwrap();
                server.send_message(*client, DefaultChannel::ReliableOrdered, message);
            }

            commands.insert_resource(Player::new(0)); // Host is always player 0
            next_game_state.set(GameState::Game);
        }
        MenuBtn::BackToMenu => {
            if let Some(client) = client.as_mut() {
                client.disconnect();
            } else if let Some(mut server) = server {
                server.disconnect_all();
                commands.remove_resource::<RenetServer>();
                commands.remove_resource::<NetcodeServerTransport>();
            }

            next_game_state.set(GameState::Menu)
        }
        MenuBtn::Quit => std::process::exit(0),
    }
}

pub fn spawn_menu_button(parent: &mut ChildBuilder, btn: MenuBtn, assets: &Local<WorldAssets>) {
    parent
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Px(350.),
                height: Val::Px(80.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Px(15.)),
                padding: UiRect::all(Val::Px(15.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON.into()),
            btn.clone(),
        ))
        .observe(recolor::<Pointer<Over>>(HOVERED_BUTTON))
        .observe(recolor::<Pointer<Out>>(NORMAL_BUTTON))
        .observe(recolor::<Pointer<Down>>(PRESSED_BUTTON))
        .observe(recolor::<Pointer<Up>>(HOVERED_BUTTON))
        .observe(on_click_menu_button)
        .with_children(|parent| {
            parent.spawn(add_text(btn.to_title(), assets));
        });
}
