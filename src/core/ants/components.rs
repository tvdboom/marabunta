use crate::core::map::components::Loc;
use bevy::prelude::*;

#[derive(Debug)]
pub enum Ant {
    BlackAnt,
    BlackQueen,
}

#[derive(Clone, Debug)]
pub enum Action {
    Idle,
    Walk(Loc),   // Location to walk to
    Dig(Entity), // Entity of the tile to dig
}

impl Action {
    pub fn interval(&self) -> f32 {
        match &self {
            Action::Idle => 0.2,
            Action::Walk(_) => 0.2,
            Action::Dig(_) => 0.05,
        }
    }
}

#[derive(Component)]
pub struct AnimationCmp {
    pub timer: Timer,
    pub last_index: usize,
    pub action: Action,
}

#[derive(Component)]
pub struct AntCmp {
    pub kind: Ant,
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
                kind: Ant::BlackAnt,
                health: 10.,
                speed: 20.,
                scale: 0.03,
                action: Action::Idle,
                z_score: 0.1,
            },
            Ant::BlackQueen => Self {
                kind: Ant::BlackQueen,
                health: 1000.,
                speed: 10.,
                scale: 0.05,
                action: Action::Idle,
                z_score: 0.9,
            },
        }
    }
}
