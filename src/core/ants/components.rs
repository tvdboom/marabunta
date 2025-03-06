use crate::core::constants::DEFAULT_WALK_SPEED;
use crate::core::map::loc::Loc;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Component)]
pub struct AntHealthWrapperCmp;

#[derive(Component)]
pub struct AntHealthCmp;

#[derive(Component)]
pub struct LeafCarryCmp;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Behavior {
    Attack,
    Brood,
    Dig,
    Harvest,
    Wander,
}

#[derive(EnumIter, Clone, Debug, Eq, PartialEq)]
pub enum Animation {
    Bite,
    Die,
    LookAround,
    Idle,
    Pinch,
    Sting,
    Walk,
    WalkPincing,
}

#[derive(EnumIter, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ant {
    BlackQueen,
    BlackAnt,
    BlackBullet,
    BlackSoldier,
    GoldTail,
    TrapJaw,
    BlackScorpion,
}

impl Ant {
    pub fn is_monster(&self) -> bool {
        match self {
            Ant::BlackScorpion => true,
            _ => false,
        }
    }

    pub fn size(&self) -> UVec2 {
        match self {
            Ant::BlackQueen => UVec2::new(307, 525),
            Ant::BlackAnt => UVec2::new(307, 438),
            Ant::BlackBullet => UVec2::new(307, 474),
            Ant::BlackSoldier => UVec2::new(367, 508),
            Ant::GoldTail => UVec2::new(466, 623),
            Ant::TrapJaw => UVec2::new(513, 577),
            Ant::BlackScorpion => UVec2::new(675, 785),
        }
    }

    pub fn all_animations(&self) -> Vec<Animation> {
        let exclude_animations = match self {
            Ant::BlackScorpion => vec![Animation::Bite, Animation::LookAround],
            _ => vec![Animation::Pinch, Animation::Sting, Animation::WalkPincing],
        };

        Animation::iter()
            .filter(|a| !exclude_animations.contains(a))
            .collect()
    }

    pub fn frames(&self, animation: &Animation) -> u32 {
        match self {
            Ant::BlackScorpion => match animation {
                Animation::Die => 5,
                Animation::Idle => 24,
                Animation::Pinch | Animation::Sting => 10,
                Animation::Walk | Animation::WalkPincing => 16,
                _ => unreachable!(),
            },
            _ => match animation {
                Animation::Bite | Animation::Walk => 8,
                Animation::Die => 10,
                Animation::LookAround | Animation::Idle => 20,
                _ => unreachable!(),
            },
        }
    }

    pub fn interval(&self, animation: &Animation) -> f32 {
        1. / self.frames(animation) as f32
    }
}

#[derive(Component)]
pub struct AnimationCmp {
    /// Animation to run
    pub animation: Animation,

    /// Repeating timer that determines the time interval between frames
    pub timer: Timer,

    /// Index of the last frame in the animation
    pub last_index: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Attack(Uuid), // Id of the target to attack
    Brood(Timer), // Brooding time remaining
    Die(Timer),   // Time the body remains visible
    Dig(Tile),    // Tile to dig
    Harvest,
    Idle,
    TargetedWalk(Uuid), // Id of the target to walk to
    Walk(Loc),          // Location to walk to
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AntCmp {
    /// Unique id across players (not entity)
    pub id: Uuid,

    /// Ant type
    pub kind: Ant,

    /// Key used to create this ant
    pub key: Option<KeyCode>,

    /// Player id of the ant's owner
    pub owner: ClientId,

    /// Scale factor of `Transform`
    /// Determines the size of the ant's sprite
    pub scale: f32,

    /// Z-score above the base ant's default z-score (0.0-0.9)
    pub z_score: f32,

    /// Food necessary to spawn the ant
    pub price: f32,

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
}

impl Default for AntCmp {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: Ant::BlackAnt,
            key: None,
            owner: 0,
            scale: 0.03,
            z_score: 0.9,
            price: 0.,
            health: 0.,
            max_health: 0.,
            speed: DEFAULT_WALK_SPEED,
            behavior: Behavior::Attack,
            action: Action::Idle,
            hatch_time: 0.,
            carry: 0.,
            max_carry: 10.,
        }
    }
}

impl AntCmp {
    pub fn new(kind: &Ant, id: ClientId) -> Self {
        match kind {
            Ant::BlackAnt => Self {
                kind: Ant::BlackAnt,
                key: Some(KeyCode::KeyZ),
                owner: id,
                z_score: 0.1,
                price: 30.,
                health: 10.,
                max_health: 10.,
                behavior: Behavior::Harvest,
                action: Action::Idle,
                hatch_time: 5.,
                max_carry: 30.,
                ..default()
            },
            Ant::BlackBullet => Self {
                kind: Ant::BlackBullet,
                key: Some(KeyCode::KeyX),
                owner: id,
                z_score: 0.2,
                price: 100.,
                health: 10.,
                max_health: 10.,
                speed: DEFAULT_WALK_SPEED + 10.,
                behavior: Behavior::Dig,
                action: Action::Idle,
                hatch_time: 10.,
                ..default()
            },
            Ant::BlackSoldier => Self {
                kind: Ant::BlackSoldier,
                key: Some(KeyCode::KeyC),
                owner: id,
                scale: 0.04,
                z_score: 0.5,
                price: 150.,
                health: 50.,
                max_health: 50.,
                speed: DEFAULT_WALK_SPEED + 5.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 15.,
                ..default()
            },
            Ant::BlackQueen => Self {
                kind: Ant::BlackQueen,
                owner: id,
                scale: 0.06,
                price: 1000.,
                health: 1000.,
                max_health: 1000.,
                speed: DEFAULT_WALK_SPEED - 2.,
                behavior: Behavior::Brood,
                action: Action::Idle,
                hatch_time: 30.,
                ..default()
            },
            Ant::GoldTail => Self {
                kind: Ant::GoldTail,
                key: Some(KeyCode::KeyV),
                owner: id,
                scale: 0.04,
                z_score: 0.6,
                price: 200.,
                health: 50.,
                max_health: 50.,
                speed: DEFAULT_WALK_SPEED,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 12.,
                ..default()
            },
            Ant::TrapJaw => Self {
                kind: Ant::TrapJaw,
                key: Some(KeyCode::KeyB),
                owner: id,
                scale: 0.05,
                z_score: 0.7,
                price: 250.,
                health: 100.,
                max_health: 100.,
                speed: DEFAULT_WALK_SPEED - 5.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                hatch_time: 20.,
                ..default()
            },
            Ant::BlackScorpion => Self {
                kind: Ant::BlackScorpion,
                owner: id,
                scale: 0.05,
                health: 100.,
                max_health: 100.,
                speed: DEFAULT_WALK_SPEED - 5.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                ..default()
            },
        }
    }

    pub fn size(&self) -> Vec2 {
        self.kind.size().as_vec2()
    }

    pub fn scaled_size(&self) -> Vec2 {
        self.size() * self.scale
    }

    pub fn animation(&self) -> Animation {
        match self.action {
            Action::Attack(_) => match self.kind {
                Ant::BlackScorpion => Animation::Sting,
                _ => Animation::Bite,
            },
            Action::Die(_) => Animation::Die,
            Action::Harvest | Action::Dig(_) => Animation::LookAround,
            Action::Brood(_) | Action::Idle => Animation::Idle,
            Action::TargetedWalk(_) | Action::Walk(_) => Animation::Walk,
        }
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
