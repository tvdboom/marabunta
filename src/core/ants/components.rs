use crate::core::map::components::Loc;
use bevy::prelude::*;

#[derive(Debug)]
pub enum Ant {
    BlackAnt,
    BlackQueen,
}

pub enum Action {
    /// Location to wander to
    Wander(Option<Loc>),

    /// Location to dig
    Dig(Option<Loc>),
}

impl Action {
    pub fn get_interval(&self) -> f32 {
        match self {
            Action::Wander(_) => 0.2,
            Action::Dig(_) => 0.2,
        }
    }
}

#[derive(Component)]
pub struct AnimationCmp {
    pub timer: Timer,
    pub last_index: usize,
}

#[derive(Component)]
pub struct AntCmp {
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
                health: 10.,
                speed: 20.,
                scale: 0.03,
                action: Action::Wander(None),
                z_score: 0.1,
            },
            Ant::BlackQueen => Self {
                health: 1000.,
                speed: 10.,
                scale: 0.05,
                action: Action::Wander(None),
                z_score: 0.9,
            },
        }
    }
}
