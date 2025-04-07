use crate::core::ants::components::{AntCmp, Egg, Owned};
use crate::core::ants::events::{DespawnAntEv, SpawnAntEv, SpawnEggEv};
use crate::core::game_settings::GameSettings;
use crate::core::map::map::Map;
use crate::core::network::{ClientMessage, ClientSendMessage, ServerMessage, ServerSendMessage};
use crate::core::persistence::Population;
use crate::core::player::Players;
use crate::core::states::GameState;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use bimap::BiMap;

#[derive(Resource, Default)]
pub struct EntityMap(pub BiMap<Entity, Entity>);

#[derive(Event)]
pub struct UpdatePopulationEv(pub Population);

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
    map: Res<Map>,
) {
    for id in server.clients_id().iter() {
        server_send_message.send(ServerSendMessage {
            message: ServerMessage::Status {
                speed: game_settings.speed,
                map: map.clone(),
                population: Population {
                    ants: ant_q
                        .iter()
                        .filter_map(|(e, t, a)| {
                            (a.team != *id).then_some((e, (t.clone(), a.clone())))
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
    ant_q: Query<(Entity, &Transform, &AntCmp)>,
    egg_q: Query<(Entity, &Transform, &Egg)>,
    players: Res<Players>,
    map: Res<Map>,
) {
    client_send_message.send(ClientSendMessage {
        message: ClientMessage::Status {
            map: map.clone(),
            population: Population {
                ants: ant_q
                    .iter()
                    .filter_map(|(e, t, a)| {
                        (a.team == players.main_id()).then_some((e, (t.clone(), a.clone())))
                    })
                    .collect(),
                eggs: egg_q
                    .iter()
                    .filter_map(|(e, t, egg)| {
                        (egg.team == players.main_id()).then_some((e, (t.clone(), egg.clone())))
                    })
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
    for UpdatePopulationEv(population) in update_population_ev.read() {
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
