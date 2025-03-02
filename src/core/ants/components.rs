use crate::core::map::loc::Loc;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Component)]
pub struct AntHealthWrapperCmp;

#[derive(Component)]
pub struct AntHealthCmp;

#[derive(Component)]
pub struct LeafCarryCmp;

#[derive(EnumIter, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ant {
    BlackAnt,
    BlackBullet,
    BlackSoldier,
    BlackQueen,
    GoldTail,
    TrapJaw,
}

impl Ant {
    pub fn size(&self) -> UVec2 {
        match self {
            Ant::BlackAnt => UVec2::new(307, 438),
            Ant::BlackBullet => UVec2::new(307, 474),
            Ant::BlackSoldier => UVec2::new(367, 508),
            Ant::BlackQueen => UVec2::new(307, 525),
            Ant::GoldTail => UVec2::new(466, 623),
            Ant::TrapJaw => UVec2::new(513, 577),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Behavior {
    Attack,
    Brood,
    Dig,
    Harvest,
    Wander,
}

#[derive(EnumIter, Debug)]
pub enum Animation {
    Bite,
    Die,
    LookAround,
    Idle,
    Walk,
}

impl Animation {
    pub fn frames(&self) -> u32 {
        match &self {
            Animation::Bite => 8,
            Animation::Die => 10,
            Animation::LookAround => 20,
            Animation::Idle => 20,
            Animation::Walk => 8,
        }
    }

    pub fn interval(&self) -> f32 {
        1. / self.frames() as f32
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Bite,
    Die,
    Dig(Tile), // Tile to dig
    Harvest,
    Idle,
    TargetedWalk(Uuid), // Id of the target to walk to
    Walk(Loc),          // Location to walk to
}

impl Action {
    pub fn animation(&self) -> Animation {
        match &self {
            Action::Bite => Animation::Bite,
            Action::Die => Animation::Die,
            Action::Harvest | Action::Dig(_) => Animation::LookAround,
            Action::Idle => Animation::Idle,
            Action::TargetedWalk(_) | Action::Walk(_) => Animation::Walk,
        }
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

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AntCmp {
    /// Unique id across players (not entity)
    pub id: Uuid,

    /// Ant type
    pub kind: Ant,

    /// Player id of the ant's owner
    pub owner: ClientId,

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

    /// Default behavior of the ant
    pub behavior: Behavior,

    /// Current action performed by the ant
    pub action: Action,

    /// Time to hatch from an egg
    pub hatch_time: f32,

    /// Current resource carry capacity
    pub carry: f32,

    /// Maximum resource carry capacity
    pub max_carry: f32,

    /// General purpose timer used for brooding, death, etc...
    pub timer: Option<Timer>,
}

impl AntCmp {
    pub fn new(kind: &Ant, id: ClientId) -> Self {
        match kind {
            Ant::BlackAnt => Self {
                id: Uuid::new_v4(),
                kind: Ant::BlackAnt,
                owner: id,
                scale: 0.03,
                z_score: 0.1,
                health: 10.,
                max_health: 10.,
                speed: 20.,
                behavior: Behavior::Harvest,
                action: Action::Idle,
                hatch_time: 5.,
                carry: 0.,
                max_carry: 30.,
                timer: None,
            },
            Ant::BlackBullet => Self {
                id: Uuid::new_v4(),
                kind: Ant::BlackBullet,
                owner: id,
                scale: 0.03,
                z_score: 0.2,
                health: 10.,
                max_health: 10.,
                speed: 30.,
                behavior: Behavior::Dig,
                action: Action::Idle,
                hatch_time: 10.,
                carry: 0.,
                max_carry: 10.,
                timer: None,
            },
            Ant::BlackSoldier => Self {
                id: Uuid::new_v4(),
                kind: Ant::BlackSoldier,
                owner: id,
                scale: 0.04,
                z_score: 0.5,
                health: 50.,
                max_health: 50.,
                speed: 15.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 15.,
                carry: 0.,
                max_carry: 10.,
                timer: None,
            },
            Ant::BlackQueen => Self {
                id: Uuid::new_v4(),
                kind: Ant::BlackQueen,
                owner: id,
                scale: 0.06,
                z_score: 0.9,
                health: 1000.,
                max_health: 1000.,
                speed: 20.,
                behavior: Behavior::Brood,
                action: Action::Idle,
                hatch_time: 30.,
                carry: 0.,
                max_carry: 1000.,
                timer: None,
            },
            Ant::GoldTail => Self {
                id: Uuid::new_v4(),
                kind: Ant::GoldTail,
                owner: id,
                scale: 0.04,
                z_score: 0.6,
                health: 50.,
                max_health: 50.,
                speed: 20.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 12.,
                carry: 0.,
                max_carry: 10.,
                timer: None,
            },
            Ant::TrapJaw => Self {
                id: Uuid::new_v4(),
                kind: Ant::TrapJaw,
                owner: id,
                scale: 0.05,
                z_score: 0.7,
                health: 100.,
                max_health: 100.,
                speed: 15.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 20.,
                carry: 0.,
                max_carry: 10.,
                timer: None,
            },
        }
    }

    pub fn size(&self) -> Vec2 {
        self.kind.size().as_vec2()
    }

    pub fn scaled_size(&self) -> Vec2 {
        self.size() * self.scale
    }
}

#[derive(Component)]
pub struct Egg {
    /// Type of ant in the egg
    pub ant: Ant,

    /// Player id of the egg's owner
    pub owner: ClientId,

    /// Time to hatch
    pub timer: Timer,
}
