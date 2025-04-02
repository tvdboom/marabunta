use crate::core::ants::components::{Action, Ant, AntCmp, Corpse};
use crate::core::ants::events::SpawnAntEv;
use crate::core::ants::utils::transform_ant;
use crate::core::audio::PlayAudioEv;
use crate::core::player::Players;
use crate::core::resources::Resources;
use crate::core::states::GameState;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use rand::prelude::IteratorRandom;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Trait {
    Alate,
    Breeding,
    Corpses,
    DoubleQueen,
    EnhancedSoldiers,
    EnhancedWarriors,
    Harvest,
    Haste,
    HealingQueen,
    Influx,
    Mastodon,
    MegaColony,
    Metamorfosis,
    Necromancer,
    ScorpionKiller,
    SuddenArmy,
    SuperQueen,
    TermiteKiller,
    Tunneling,
    WanderingQueen,
    Warlike,
    WaspKiller,
}

#[derive(Clone)]
pub struct TraitCmp {
    pub kind: Trait,
    pub image: String,
    pub description: String,
}

impl TraitCmp {
    pub fn new(kind: &Trait) -> Self {
        match kind {
            Trait::Alate => Self {
                kind: Trait::Alate,
                image: "alate".to_string(),
                description: "\
                    Unlocks the alate (flying) ants. Alates are incredibly fast and powerful \
                    ants that can turn the tie of any war."
                    .to_string(),
            },
            Trait::Breeding => Self {
                kind: Trait::Breeding,
                image: "eggs".to_string(),
                description: "\
                    Eggs hatch twice as fast and have double the health. Enhance your colony's \
                    growth by increasing the larva production rate."
                    .to_string(),
            },
            Trait::Corpses => Self {
                kind: Trait::Corpses,
                image: "corpses".to_string(),
                description: "\
                    Corpses lie twice as long on the ground, allowing more nutrient collection \
                    and the healing of your ants."
                    .to_string(),
            },
            Trait::DoubleQueen => Self {
                kind: Trait::DoubleQueen,
                image: "double-queen".to_string(),
                description: "\
                    Your colony gains an extra queen. The queens cooperate, increasing egg \
                    production and colony growth. Both queens need to die to lose the game."
                    .to_string(),
            },
            Trait::EnhancedSoldiers => Self {
                kind: Trait::EnhancedSoldiers,
                image: "soldiers".to_string(),
                description: "\
                    Soldier ants increase their damage and speed. Use this trait to create a \
                    powerful army."
                    .to_string(),
            },
            Trait::EnhancedWarriors => Self {
                kind: Trait::EnhancedWarriors,
                image: "battle".to_string(),
                description: "\
                    Warriors ants increase their damage and health. Use this trait to create \
                    a powerful army."
                    .to_string(),
            },
            Trait::Harvest => Self {
                kind: Trait::Harvest,
                image: "harvest".to_string(),
                description: "\
                    Your workers harvest resources twice as fast. Resources are the lifeblood \
                    of the colony. More leaves and nutrients means more and stronger ants."
                    .to_string(),
            },
            Trait::Haste => Self {
                kind: Trait::Haste,
                image: "haste".to_string(),
                description: "\
                    All your ants move 20% faster. Speed is the key to productivity. Faster \
                    ants means faster food collection and reaching the enemy earlier."
                    .to_string(),
            },
            Trait::HealingQueen => Self {
                kind: Trait::HealingQueen,
                image: "healing".to_string(),
                description: "\
                    Your queen can heal her wounds. If not under attack, the queen regenerates \
                    over time remaining idle. The game is lost if the queen dies, so a healthy \
                    queen is paramount."
                    .to_string(),
            },
            Trait::Influx => Self {
                kind: Trait::Influx,
                image: "influx".to_string(),
                description: "\
                    Immediately receive a large amount of leaves and nutrients."
                    .to_string(),
            },
            Trait::Mastodon => Self {
                kind: Trait::Mastodon,
                image: "mastodon".to_string(),
                description: "\
                    Unlocks the mastodon ants. Enormous ants with powerful jaws that deal tons \
                    of damage."
                    .to_string(),
            },
            Trait::MegaColony => Self {
                kind: Trait::MegaColony,
                image: "megacolony".to_string(),
                description: "\
                    All your ants cost 10% less food to produce. Quickly become the largest \
                    colony around and overcome your enemies by the sheer numbers."
                    .to_string(),
            },
            Trait::Metamorfosis => Self {
                kind: Trait::Metamorfosis,
                image: "metamorfosis".to_string(),
                description: "\
                    All your workers turn into soldiers. This is a one-time transformation \
                    for the current workers. Queued ants remain the same."
                    .to_string(),
            },
            Trait::Necromancer => Self {
                kind: Trait::Necromancer,
                image: "necromancer".to_string(),
                description: "\
                    All the current corpses of your ants come back to live with full health. \
                    This includes the queen if you were about to lose the game!"
                    .to_string(),
            },
            Trait::ScorpionKiller => Self {
                kind: Trait::ScorpionKiller,
                image: "scorpion".to_string(),
                description: "\
                    All your ants have double the damage against scorpions. Scorpions are \
                    dangerous enemies, often encountered by excavators when digging tunnels."
                    .to_string(),
            },
            Trait::SuddenArmy => Self {
                kind: Trait::SuddenArmy,
                image: "sudden-army".to_string(),
                description: "\
                    A random number of soldier and warrior ants immediately spawn around your \
                    queen. Surprise your enemies with a sudden army."
                    .to_string(),
            },
            Trait::SuperQueen => Self {
                kind: Trait::SuperQueen,
                image: "super-queen".to_string(),
                description: "\
                    The queen increases in health and strength, but walks slower. If you \
                    have more than one queen, they all gain the bonuses."
                    .to_string(),
            },
            Trait::TermiteKiller => Self {
                kind: Trait::TermiteKiller,
                image: "termites".to_string(),
                description: "\
                    All your ants have double the damage against termites. Termites attack \
                    in groups."
                    .to_string(),
            },
            Trait::Tunneling => Self {
                kind: Trait::Tunneling,
                image: "tunneling".to_string(),
                description: "\
                    Excavator ants dig twice as fast. A rapid expansion of the nest means \
                    discovering more food sources, but also encountering enemies faster."
                    .to_string(),
            },
            Trait::WanderingQueen => Self {
                kind: Trait::WanderingQueen,
                image: "wandering".to_string(),
                description: "\
                    The queen is able to move outside the base. It can lay eggs anywhere and \
                    her default behavior becomes wandering around the map. The player can give \
                    commands to the queen."
                    .to_string(),
            },
            Trait::Warlike => Self {
                kind: Trait::Warlike,
                image: "workers".to_string(),
                description: "\
                    Your workers become stronger, gaining twice the health and damage, but \
                    reducing their harvesting speed by half."
                    .to_string(),
            },
            Trait::WaspKiller => Self {
                kind: Trait::WaspKiller,
                image: "wasp".to_string(),
                description: "\
                    All your ants have double the damage against wasps. Wasps sometimes enter \
                    the tunnels through chambers with holes that lead to the surface."
                    .to_string(),
            },
        }
    }
}

#[derive(Event)]
pub struct TraitSelectedEv {
    pub selected: Trait,
}

pub fn select_trait_event(
    mut commands: Commands,
    mut ant_q: Query<(Entity, &mut Transform, &mut AntCmp)>,
    mut trait_selected_ev: EventReader<TraitSelectedEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut play_audio_ev: EventWriter<PlayAudioEv>,
    mut players: ResMut<Players>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for ev in trait_selected_ev.read() {
        play_audio_ev.send(PlayAudioEv::new("button"));

        for player in players.0.iter_mut().filter(|p| p.id != ClientId::MAX) {
            let selected = if player.id == 0 {
                ev.selected.clone()
            } else {
                // For NPCs, select a random trait they don't have
                Trait::iter()
                    .filter(|t| !player.has_trait(&t))
                    .choose(&mut rng())
                    .unwrap()
            };

            player.traits.push(selected);

            match selected {
                Trait::DoubleQueen => {
                    spawn_ant_ev.send(SpawnAntEv {
                        ant: AntCmp::new(&Ant::Queen, &player),
                        transform: Transform {
                            // Spawn the queen where the current one is located
                            translation: ant_q
                                .iter()
                                .find(|(_, _, a)| a.kind == Ant::Queen && a.team == player.id)
                                .unwrap()
                                .1
                                .translation,
                            rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                            ..default()
                        },
                    });
                }
                Trait::EnhancedSoldiers => {
                    let soldier = AntCmp::new(&Ant::Soldier, &player);
                    ant_q
                        .iter_mut()
                        .filter(|(_, _, a)| a.kind == Ant::Soldier && a.team == player.id)
                        .for_each(|(_, mut t, mut a)| transform_ant(&mut t, &mut a, &soldier));
                }
                Trait::EnhancedWarriors => {
                    let warrior = AntCmp::new(&Ant::Warrior, &player);
                    ant_q
                        .iter_mut()
                        .filter(|(_, _, a)| a.kind == Ant::Warrior && a.team == player.id)
                        .for_each(|(_, mut t, mut a)| transform_ant(&mut t, &mut a, &warrior));
                }
                Trait::Influx => {
                    player.resources += &Resources::new(
                        rng().random_range(1e3..2e3),
                        rng().random_range(100.0..300.),
                    );
                }
                Trait::Metamorfosis => {
                    let soldier = AntCmp::new(&Ant::Soldier, &player);
                    ant_q
                        .iter_mut()
                        .filter(|(_, _, a)| {
                            a.kind == Ant::Worker && a.team == player.id && a.health > 0.
                        })
                        .for_each(|(_, mut t, mut a)| transform_ant(&mut t, &mut a, &soldier));
                }
                Trait::Necromancer => {
                    ant_q
                        .iter_mut()
                        .filter(|(_, _, a)| a.team == player.id && a.health == 0.)
                        .for_each(|(e, _, mut a)| {
                            a.health = a.max_health;
                            a.command = None;
                            a.action = Action::Idle;
                            commands.entity(e).remove::<Corpse>();
                        });
                }
                Trait::SuddenArmy => {
                    for _ in 0..rng().random_range(8..15) {
                        let ant = if rng().random::<f32>() < 0.5 {
                            Ant::Soldier
                        } else {
                            Ant::Warrior
                        };

                        spawn_ant_ev.send(SpawnAntEv {
                            ant: AntCmp::new(&ant, &player),
                            transform: Transform {
                                // Spawn the queen where the current one is located
                                translation: ant_q
                                    .iter()
                                    .find(|(_, _, a)| a.kind == Ant::Queen && a.team == player.id)
                                    .unwrap()
                                    .1
                                    .translation,
                                rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                                ..default()
                            },
                        });
                    }
                }
                Trait::SuperQueen => {
                    let queen = AntCmp::new(&Ant::Queen, &player);
                    ant_q
                        .iter_mut()
                        .filter(|(_, _, a)| a.kind == Ant::Queen && a.team == player.id)
                        .for_each(|(_, mut t, mut a)| transform_ant(&mut t, &mut a, &queen));
                }
                Trait::Warlike => {
                    let worker = AntCmp::new(&Ant::Worker, &player);
                    ant_q
                        .iter_mut()
                        .filter(|(_, _, a)| a.kind == Ant::Worker && a.team == player.id)
                        .for_each(|(_, mut t, mut a)| transform_ant(&mut t, &mut a, &worker));
                }
                _ => (),
            }
        }

        next_game_state.set(GameState::Running);
    }
}
