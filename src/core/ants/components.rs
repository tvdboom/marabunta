use crate::core::map::loc::Loc;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use strum_macros::EnumIter;

#[derive(Component)]
pub struct AntHealthWrapper(pub Entity);

#[derive(Component)]
pub struct AntHealth;

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

#[derive(EnumIter, Clone, Debug, PartialEq)]
pub enum Action {
    Bite,
    Die,
    Idle,
    Walk(Loc), // Location to walk to
    Dig(Tile), // Tile to dig
}

impl Action {
    pub fn frames(&self) -> u32 {
        match &self {
            Action::Bite => 8,
            Action::Die => 10,
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
    /// Action corresponding to this animation
    pub action: Action,

    /// Repeating timer that determines the time interval between frames
    pub timer: Timer,

    /// Index of the last frame in the animation
    pub last_index: usize,
}

#[derive(Component, Clone)]
pub struct AntCmp {
    /// Ant type
    pub kind: Ant,

    /// Scale factor of `Transform`
    /// Determines the size of the ant's sprite
    pub scale: f32,

    /// Z-score above the base ant's default z-score (0.0-0.9)
    pub z_score: f32,

    /// Current health
    pub health: f32,

    /// Maximum health
    pub max_health: f32,

    /// Speed in pixels per second
    pub speed: f32,

    /// Current action performed by the ant
    pub action: Action,

    /// Time to hatch from an egg
    pub hatch_time: f32,

    /// General purpose timer used for brooding, death, etc...
    pub timer: Option<Timer>,
}

impl AntCmp {
    pub fn new(kind: Ant) -> Self {
        match kind {
            Ant::BlackAnt => Self {
                kind: Ant::BlackAnt,
                scale: 0.03,
                z_score: 0.1,
                health: 10.,
                max_health: 10.,
                speed: 20.,
                action: Action::Idle,
                hatch_time: 5.,
                timer: None,
            },
            Ant::BlackQueen => Self {
                kind: Ant::BlackQueen,
                scale: 0.05,
                z_score: 0.9,
                health: 1000.,
                max_health: 1000.,
                speed: 20.,
                action: Action::Idle,
                hatch_time: 30.,
                timer: None,
            },
        }
    }

    pub fn size(&self) -> Vec2 {
        self.kind.size().as_vec2() * self.scale
    }
}

#[derive(Component)]
pub struct Egg {
    /// Type of ant in the egg
    pub ant: Ant,

    /// Time to hatch
    pub timer: Timer,
}
