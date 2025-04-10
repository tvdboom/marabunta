use crate::core::ants::components::{Action, AntCmp, Egg, Owned};
use crate::core::ants::events::{DespawnAntEv, SpawnAntEv, SpawnEggEv};
use crate::core::game_settings::GameSettings;
use crate::core::network::{ClientMessage, ClientSendMessage, ServerMessage, ServerSendMessage};
use crate::core::persistence::Population;
use crate::core::player::Players;
use crate::core::states::GameState;
use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use bimap::BiMap;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct EntityMap(pub BiMap<Entity, Entity>);

#[derive(Event)]
pub struct UpdatePopulationEv {
    pub population: Population,
    pub id: ClientId,
}

pub fn update_game_state(
    mut server_send_message: EventWriter<ServerSendMessage>,
    mut client_send_message: EventWriter<ClientSendMessage>,
    game_state: Res<State<GameState>>,
) {
    server_send_message.send(ServerSendMessage {
        message: ServerMessage::State(*game_state.get()),
        client: None,
    });

    client_send_message.send(ClientSendMessage {
        message: ClientMessage::State(*game_state.get()),
    });
}

pub fn server_send_status(
    mut server_send_message: EventWriter<ServerSendMessage>,
    server: Res<RenetServer>,
    ant_q: Query<(Entity, &Transform, &AntCmp)>,
    egg_q: Query<(Entity, &Transform, &Egg)>,
    game_settings: Res<GameSettings>,
    entity_map: Res<EntityMap>,
) {
    for id in server.clients_id().iter() {
        server_send_message.send(ServerSendMessage {
            message: ServerMessage::Status {
                speed: game_settings.speed,
                population: Population {
                    ants: ant_q
                        .iter()
                        .filter(|(_, _, a)| a.team != *id)
                        .map(|(e, t, a)| {
                            let mut a = a.clone();

                            // Map attacking entity to the entity on the server
                            if let Action::Attack(e) = &mut a.action {
                                *e = *entity_map.0.get_by_right(e).unwrap_or(e);
                            }

                            (e, (t.clone(), a))
                        })
                        .collect(),
                    eggs: egg_q
                        .iter()
                        .filter_map(|(e, t, a)| {
                            (a.team != *id).then_some((e, (t.clone(), a.clone())))
                        })
                        .collect(),
                },
            },
            client: Some(*id),
        });
    }
}

pub fn client_send_status(
    mut client_send_message: EventWriter<ClientSendMessage>,
    ant_q: Query<(Entity, &Transform, &AntCmp), With<Owned>>,
    egg_q: Query<(Entity, &Transform, &Egg), With<Owned>>,
    players: Res<Players>,
    entity_map: Res<EntityMap>,
) {
    client_send_message.send(ClientSendMessage {
        message: ClientMessage::Status {
            player: players.main().clone(),
            population: Population {
                ants: ant_q
                    .iter()
                    .map(|(e, t, a)| {
                        let mut a = a.clone();

                        // Map attacking entity to the entity on the server
                        if let Action::Attack(e) = &mut a.action {
                            *e = *entity_map.0.get_by_right(e).unwrap_or(e);
                        }

                        (e, (t.clone(), a))
                    })
                    .collect(),
                eggs: egg_q
                    .iter()
                    .map(|(e, t, egg)| (e, (t.clone(), egg.clone())))
                    .collect(),
            },
        },
    });
}

pub fn update_population_event(
    mut update_population_ev: EventReader<UpdatePopulationEv>,
    mut ant_q: Query<(Entity, &mut Transform, &mut AntCmp), Without<Owned>>,
    mut egg_q: Query<(Entity, &mut Transform, &mut Egg), (Without<Owned>, Without<AntCmp>)>,
    entity_map: Res<EntityMap>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut spawn_egg_ev: EventWriter<SpawnEggEv>,
    mut despawn_ant_ev: EventWriter<DespawnAntEv>,
) {
    let mut seen = HashSet::new();
    for UpdatePopulationEv { population, id } in update_population_ev.read() {
        if !seen.insert(id) {
            continue;
        }

        // Despawn all that are not in the new population
        for (ant_e, _, _) in &ant_q {
            if !population
                .ants
                .contains_key(entity_map.0.get_by_right(&ant_e).unwrap())
            {
                despawn_ant_ev.send(DespawnAntEv { entity: ant_e });
            }
        }

        for (egg_e, _, _) in &egg_q {
            if !population
                .eggs
                .contains_key(entity_map.0.get_by_right(&egg_e).unwrap())
            {
                despawn_ant_ev.send(DespawnAntEv { entity: egg_e });
            }
        }

        // Update the current population
        for (entity, (t, a)) in population.ants.iter() {
            if let Some(ant_e) = entity_map.0.get_by_left(entity) {
                if let Ok((_, mut ant_t, mut ant)) = ant_q.get_mut(*ant_e) {
                    *ant_t = *t;
                    *ant = a.clone();
                }
            } else {
                spawn_ant_ev.send(SpawnAntEv {
                    ant: a.clone(),
                    transform: *t,
                    entity: Some(*entity),
                });
            }
        }

        for (entity, (t, e)) in population.eggs.iter() {
            if let Some(egg_e) = entity_map.0.get_by_left(entity) {
                if let Ok((_, mut egg_t, mut egg)) = egg_q.get_mut(*egg_e) {
                    *egg_t = *t;
                    *egg = e.clone();
                }
            } else {
                spawn_egg_ev.send(SpawnEggEv {
                    ant: e.ant.clone(),
                    transform: *t,
                    entity: Some(*entity),
                });
            }
        }
    }
}
