use crate::core::assets::WorldAssets;
use crate::core::constants::{
    BUTTON_TEXT_SIZE, HOVERED_BUTTON_COLOR, NORMAL_BUTTON_COLOR, PRESSED_BUTTON_COLOR,
};
use crate::core::game_settings::{GameMode, GameSettings};
use crate::core::map::systems::create_map;
use crate::core::map::ui::utils::{add_text, recolor};
use crate::core::menu::settings::AntColor;
use crate::core::network::{new_renet_client, new_renet_server, ServerMessage};
use crate::core::persistence::{LoadGameEv, SaveGameEv};
use crate::core::player::{Player, Players};
use crate::core::states::{AppState, GameState};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy_renet::netcode::{NetcodeClientTransport, NetcodeServerTransport};
use bevy_renet::renet::{DefaultChannel, RenetClient, RenetServer};

#[derive(Component)]
pub struct MenuCmp;

#[derive(Component, Clone, Debug)]
pub enum MenuBtn {
    Singleplayer,
    NewGame,
    LoadGame,
    Multiplayer,
    HostGame,
    FindGame,
    Play,
    Back,
    Continue,
    SaveGame,
    Settings,
    Quit,
}

#[derive(Component)]
pub struct LobbyTextCmp;

pub fn on_click_menu_button(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    btn_q: Query<&MenuBtn>,
    server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
    mut game_settings: ResMut<GameSettings>,
    mut load_game_ev: EventWriter<LoadGameEv>,
    mut save_game_ev: EventWriter<SaveGameEv>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    match btn_q.get(trigger.entity()).unwrap() {
        MenuBtn::Singleplayer => {
            next_app_state.set(AppState::SinglePlayerMenu);
        }
        MenuBtn::NewGame => {
            // Add the player to the resource
            let mut p = vec![Player::new(0, game_settings.color.clone())];

            // Add the NPCs to the resource
            (1..=game_settings.n_opponents)
                .for_each(|id| p.push(Player::new(id, game_settings.color.inverse())));

            // Create map before pushing default player
            commands.insert_resource(create_map(&p));

            // Add the default value used for monsters
            p.push(Player::default());

            // Update the resource
            commands.insert_resource(Players(p));

            game_settings.mode = GameMode::SinglePlayer;
            next_app_state.set(AppState::Game);
        }
        MenuBtn::LoadGame => {
            load_game_ev.send(LoadGameEv);
        }
        MenuBtn::Multiplayer => {
            next_app_state.set(AppState::MultiPlayerMenu);
        }
        MenuBtn::HostGame => {
            // Remove client resources if they exist
            if client.is_some() {
                commands.remove_resource::<RenetClient>();
                commands.remove_resource::<NetcodeClientTransport>();
            }

            let (server, transport) = new_renet_server();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_app_state.set(AppState::Lobby);
        }
        MenuBtn::FindGame => {
            let (server, transport) = new_renet_client();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_app_state.set(AppState::Lobby);
        }
        MenuBtn::Play => {
            // Multiplayer context
            let mut server = server.unwrap();

            let mut ids = vec![0];
            ids.extend(server.clients_id());

            let players = ids
                .iter()
                .map(|id| Player::new(*id, game_settings.color.inverse()))
                .collect::<Vec<_>>();

            let map = create_map(&players);

            // Send the start game signal to all clients with their player id
            for client in server.clients_id().iter() {
                let message = bincode::serialize(&ServerMessage::StartGame {
                    id: *client,
                    color: AntColor::Red,
                    settings: game_settings.clone(),
                    map: map.clone(),
                })
                .unwrap();
                server.send_message(*client, DefaultChannel::ReliableOrdered, message);
            }

            game_settings.mode = GameMode::MultiPlayer(ids);
            commands.insert_resource(Players(players));
            commands.insert_resource(map);

            next_app_state.set(AppState::Game);
        }
        MenuBtn::Back => match *app_state.get() {
            AppState::SinglePlayerMenu | AppState::MultiPlayerMenu | AppState::Settings => {
                next_app_state.set(AppState::MainMenu);
            }
            AppState::Lobby => {
                if let Some(client) = client.as_mut() {
                    client.disconnect();
                } else if let Some(mut server) = server {
                    server.disconnect_all();
                    commands.remove_resource::<RenetServer>();
                    commands.remove_resource::<NetcodeServerTransport>();
                }

                next_app_state.set(AppState::MultiPlayerMenu);
            }
            _ => unreachable!(),
        },
        MenuBtn::Continue => {
            next_game_state.set(GameState::Running);
        }
        MenuBtn::SaveGame => {
            save_game_ev.send(SaveGameEv);
        }
        MenuBtn::Settings => {
            next_app_state.set(AppState::Settings);
        }
        MenuBtn::Quit => match *app_state.get() {
            AppState::Game => {
                next_game_state.set(GameState::default());
                next_app_state.set(AppState::MainMenu)
            }
            AppState::MainMenu => std::process::exit(0),
            _ => unreachable!(),
        },
    }
}

pub fn spawn_menu_button(
    parent: &mut ChildBuilder,
    btn: MenuBtn,
    assets: &WorldAssets,
    window: &Window,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(25.),
                height: Val::Percent(10.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Percent(1.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR.into()),
            btn.clone(),
        ))
        .observe(recolor::<Pointer<Over>>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Pointer<Out>>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Pointer<Down>>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Pointer<Up>>(HOVERED_BUTTON_COLOR))
        .observe(on_click_menu_button)
        .with_children(|parent| {
            parent.spawn(add_text(
                btn.to_title(),
                "bold",
                BUTTON_TEXT_SIZE,
                assets,
                window,
            ));
        });
}
