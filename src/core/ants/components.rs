use crate::core::map::components::Loc;
use crate::utils::NameFromEnum;
use bevy::prelude::*;

#[derive(Debug)]
pub enum Ant {
    BlackAnt,
    BlackQueen,
}

#[derive(Clone, Debug)]
pub enum Action {
    Idle,
    Walk(Loc),
    Dig,
}

#[derive(Component)]
pub struct AnimationCmp {
    pub timer: Timer,
    pub last_index: usize,
    pub action: Action,
}

#[derive(Component)]
pub struct AntCmp {
    pub name: String,
    pub health: f32,
    pub speed: f32,
    pub scale: f32,
    pub action: Action,
    pub z_score: f32, // 0.0 - 0.9 above base ant z-score
}

impl AntCmp {
    pub fn new(kind: Ant) -> Self {
        match kind {
            Ant::BlackAnt => Self {
                name: Ant::BlackAnt.to_snake(),
                health: 10.,
                speed: 20.,
                scale: 0.03,
                action: Action::Idle,
                z_score: 0.1,
            },
            Ant::BlackQueen => Self {
                name: Ant::BlackQueen.to_snake(),
                health: 1000.,
                speed: 10.,
                scale: 0.05,
                action: Action::Idle,
                z_score: 0.9,
            },
        }
    }
}
