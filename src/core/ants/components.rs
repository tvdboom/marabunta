use crate::core::map::components::Loc;
use bevy::prelude::*;

pub enum Ant {
    BlackQueen,
}

pub enum Action {
    Move,
}

impl Action {
    pub fn get_interval(&self) -> f32 {
        match self {
            Action::Move => 0.1,
        }
    }
}

pub enum Movement {
    Wander(Option<Vec<Loc>>),
}

#[derive(Component)]
pub struct AnimationCmp {
    pub timer: Timer,
    pub last_index: usize,
}

#[derive(Component)]
pub struct AntCmp {
    pub health: f32,
    pub scale: f32,
    pub rotation: f32,
    pub loc: Loc,
    pub action: Action,
    pub movement: Movement,
}

impl AntCmp {
    pub fn new(kind: Ant, loc: Loc) -> Self {
        match kind {
            Ant::BlackQueen => Self {
                health: 100.,
                scale: 0.05,
                rotation: 0.,
                loc,
                action: Action::Move,
                movement: Movement::Wander(None),
            },
        }
    }
}
