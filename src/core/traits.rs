use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::SpawnAntEv;
use crate::core::ants::utils::transform_ant;
use crate::core::assets::WorldAssets;
use crate::core::player::Player;
use crate::core::states::GameState;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};
use rand::{rng, Rng};
use std::f32::consts::PI;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Trait {
    Alate,
    Breeding,
    DoubleQueen,
    EnhancedSoldiers,
    Harvest,
    Haste,
    HealingQueen,
    Mastodon,
    MegaColony,
    ScorpionKiller,
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
            Trait::Harvest => Self {
                kind: Trait::Harvest,
                image: "harvest".to_string(),
                description: "\
                    Your workers harvest food twice as fast. Food is the lifeblood of the \
                    colony. More food means more and stronger ants."
                    .to_string(),
            },
            Trait::Haste => Self {
                kind: Trait::Haste,
                image: "haste".to_string(),
                description: "\
                    All your ants move 10% faster. Speed is the key to productivity. Faster \
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
            Trait::ScorpionKiller => Self {
                kind: Trait::ScorpionKiller,
                image: "scorpion".to_string(),
                description: "\
                    All your ants have double the damage against scorpions. Scorpions are \
                    dangerous enemies, often encountered by excavators when digging tunnels."
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
                    The queen moves outside the base. It lays eggs anywhere and wander \
                    around the map."
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
pub struct TraitSelectedEv(pub Trait);

pub fn select_trait_event(
    mut ant_q: Query<(&mut Transform, &mut AntCmp)>,
    mut trait_selected_ev: EventReader<TraitSelectedEv>,
    mut spawn_ant_ev: EventWriter<SpawnAntEv>,
    mut player: ResMut<Player>,
    mut next_game_state: ResMut<NextState<GameState>>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    for ev in trait_selected_ev.read() {
        audio.play(assets.audio("button"));
        player.traits.push(ev.0.clone());

        match ev.0 {
            Trait::DoubleQueen => {
                spawn_ant_ev.send(SpawnAntEv {
                    ant: AntCmp::new(&Ant::Queen, &player),
                    transform: Transform {
                        // Spawn the queen where the current one is located
                        translation: ant_q
                            .iter()
                            .find(|(_, a)| a.kind == Ant::Queen && a.team == player.id)
                            .unwrap()
                            .0
                            .translation,
                        rotation: Quat::from_rotation_z(rng().random_range(0.0..2. * PI)),
                        ..default()
                    },
                });
            }
            Trait::SuperQueen => {
                let queen = AntCmp::new(&Ant::Queen, &player);
                ant_q
                    .iter_mut()
                    .filter(|(_, a)| a.kind == Ant::Queen && a.team == player.id)
                    .for_each(|(mut t, mut a)| transform_ant(&mut t, &mut a, &queen));
            }
            Trait::Warlike => {
                let worker = AntCmp::new(&Ant::Worker, &player);
                ant_q
                    .iter_mut()
                    .filter(|(_, a)| a.kind == Ant::Worker && a.team == player.id)
                    .for_each(|(mut t, mut a)| transform_ant(&mut t, &mut a, &worker));
            }
            Trait::EnhancedSoldiers => {
                let soldier = AntCmp::new(&Ant::Soldier, &player);
                ant_q
                    .iter_mut()
                    .filter(|(_, a)| a.kind == Ant::Worker && a.team == player.id)
                    .for_each(|(mut t, mut a)| transform_ant(&mut t, &mut a, &soldier));
            }
            _ => (),
        }

        next_game_state.set(GameState::Running);
    }
}
