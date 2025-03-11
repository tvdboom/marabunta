use crate::core::ants::components::{Ant, AntCmp};
use crate::core::ants::events::SpawnAntEv;
use crate::core::assets::WorldAssets;
use crate::core::player::Player;
use crate::core::states::GameState;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};
use rand::{rng, Rng};
use std::f32::consts::PI;
use strum_macros::EnumIter;

#[derive(EnumIter, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Trait {
    SuperQueen,
    DoubleQueen,
    Tunneling,
}

pub struct TraitCmp {
    pub kind: Trait,
    pub image: String,
    pub description: String,
}

impl TraitCmp {
    pub fn new(kind: &Trait) -> Self {
        match kind {
            Trait::DoubleQueen => Self {
                kind: Trait::DoubleQueen,
                image: "double-queen".to_string(),
                description: "\
                    Your colony gains an extra queen. The queens cooperate, increasing egg \
                    production and colony growth. Both queens need to die to lose the game".to_string(),
            },
            Trait::Tunneling => Self {
                kind: Trait::Tunneling,
                image: "tunneling".to_string(),
                description: "\
                    Excavator ants dig twice as fast. A rapid expansion of the nest means \
                    discovering more food sources, but also encountering enemies faster.".to_string(),
            },
            Trait::SuperQueen => Self {
                kind: Trait::SuperQueen,
                image: "super-queen".to_string(),
                description: "\
                    The queen increases in health and strength, but walks slower. If you \
                    have more than one queen, they all gain the bonuses.".to_string(),
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
                ant_q.iter_mut().filter(|(_, a)| a.kind == Ant::Queen && a.team == player.id).for_each(
                    |(mut t, mut a)| {
                        t.scale = Vec3::splat(queen.scale);

                        // Spawn with the same health ratio as it currently has
                        let mut queen = queen.clone();
                        queen.health = (a.health / a.max_health) * queen.max_health;
                        *a = queen;
                    },
                );
            }
            _ => (),
        }

        next_game_state.set(GameState::Running);
    }
}
