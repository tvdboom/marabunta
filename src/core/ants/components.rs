use crate::core::map::components::Loc;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug)]
pub enum Ant {
    BlackAnt,
    BlackQueen,
}

impl Ant {
    pub fn size(&self) -> UVec2 {
        match self {
            Ant::BlackAnt => UVec2::new(307, 438),
            Ant::BlackQueen => UVec2::new(307, 525),
        }
    }
}

#[derive(EnumIter, Clone, Debug)]
pub enum Action {
    Bite,
    Idle,
    Walk(Loc), // Location to walk to
    Dig(Tile), // Tile to dig
}

impl Action {
    pub fn columns(&self) -> u32 {
        match &self {
            Action::Bite => 8,
            Action::Dig(_) => 20,
            Action::Idle => 20,
            Action::Walk(_) => 8,
        }
    }

    pub fn interval(&self) -> f32 {
        match &self {
            Action::Dig(_) => 0.05,
            _ => 0.2,
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
