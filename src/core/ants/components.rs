use crate::core::map::loc::Loc;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Component)]
pub struct AntHealthWrapper(pub Entity);

#[derive(Component)]
pub struct AntHealth;

#[derive(EnumIter, Clone, Debug, Serialize, Deserialize)]
pub enum Ant {
    BlackAnt,
    BlackBullet,
    BlackSoldier,
    BlackQueen,
    GoldTail,
}

impl Ant {
    pub fn size(&self) -> UVec2 {
        match self {
            Ant::BlackAnt => UVec2::new(307, 438),
            Ant::BlackBullet => UVec2::new(307, 474),
            Ant::BlackSoldier => UVec2::new(367, 508),
            Ant::BlackQueen => UVec2::new(307, 525),
            Ant::GoldTail => UVec2::new(466, 623),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Behavior {
    Attack,
    Brood,
    Wander,
    Dig,
}

#[derive(EnumIter, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
                behavior: Behavior::Dig,
                action: Action::Idle,
                hatch_time: 5.,
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
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 10.,
                timer: None,
            },
            Ant::BlackSoldier => Self {
                id: Uuid::new_v4(),
                kind: Ant::BlackSoldier,
                owner: id,
                scale: 0.04,
                z_score: 0.6,
                health: 50.,
                max_health: 50.,
                speed: 15.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 15.,
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
                timer: None,
            },
            Ant::GoldTail => Self {
                id: Uuid::new_v4(),
                kind: Ant::GoldTail,
                owner: id,
                scale: 0.04,
                z_score: 0.7,
                health: 50.,
                max_health: 50.,
                speed: 20.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 12.,
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

    /// Player id of the egg's owner
    pub owner: ClientId,

    /// Time to hatch
    pub timer: Timer,
}
