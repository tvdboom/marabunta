use crate::core::map::components::Loc;
use bevy::prelude::*;

#[derive(Debug)]
pub enum Ant {
    BlackAnt,
    BlackQueen,
}

pub enum Action {
    Wander(Option<Vec<Loc>>),
}

impl Action {
    pub fn get_interval(&self) -> f32 {
        match self {
            Action::Wander(_) => 0.2,
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
}

impl AntCmp {
    pub fn new(kind: Ant) -> Self {
        match kind {
            Ant::BlackAnt => Self {
                health: 10.,
                speed: 20.,
                scale: 0.03,
                action: Action::Wander(None),
            },
            Ant::BlackQueen => Self {
                health: 1000.,
                speed: 10.,
                scale: 0.05,
                action: Action::Wander(None),
            },
        }
    }
}
