use crate::core::map::components::Loc;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Debug)]
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
    pub fn frames(&self) -> u32 {
        match &self {
            Action::Bite => 8,
            Action::Dig(_) => 20,
            Action::Idle => 20,
            Action::Walk(_) => 8,
        }
    }

    pub fn interval(&self) -> f32 {
        1. / self.frames() as f32
    }
}

#[derive(Component)]
pub struct AnimationCmp {
    pub timer: Timer,
    pub last_index: usize,
    pub action: Action,
}

#[derive(Component, Clone)]
pub struct AntCmp {
    pub kind: Ant,
    pub health: f32,
    pub speed: f32,
    pub scale: f32,
    pub action: Action,
    pub z_score: f32, // 0.0 - 0.9 above base ant z-score
    pub brooding: f32,
    pub brooding_timer: Option<Timer>,
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
                brooding: 5.,
                brooding_timer: None,
            },
            Ant::BlackQueen => Self {
                kind: Ant::BlackQueen,
                health: 1000.,
                speed: 10.,
                scale: 0.05,
                action: Action::Idle,
                z_score: 0.9,
                brooding: 30.,
                brooding_timer: None,
            },
        }
    }
}

#[derive(Component)]
pub struct Egg {
    pub ant: Ant,
    pub timer: Timer,
}
