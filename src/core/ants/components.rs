use crate::core::constants::DEFAULT_WALK_SPEED;
use crate::core::map::loc::Loc;
use crate::core::map::tile::Tile;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use rand::{rng, Rng};
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    Attack,
    Brood,
    Dig,
    Harvest,
    Heal,
    Wander,
}

#[derive(EnumIter, Clone, Debug, Eq, PartialEq)]
pub enum Animation {
    Bite,
    Die,
    Fly,
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
    BlackAlate,
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
            Ant::BlackAlate => UVec2::new(510, 512),
            Ant::BlackScorpion => UVec2::new(675, 785),
        }
    }

    pub fn all_animations(&self) -> Vec<Animation> {
        let exclude_animations = match self {
            Ant::BlackAlate => vec![Animation::Pinch, Animation::Sting, Animation::WalkPincing],
            Ant::BlackScorpion => vec![Animation::Bite, Animation::Fly, Animation::LookAround],
            _ => vec![
                Animation::Fly,
                Animation::Pinch,
                Animation::Sting,
                Animation::WalkPincing,
            ],
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
                Animation::Fly => 12,
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
    Heal,
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
    /// In singleplayer mode, this is always the same value
    pub owner: ClientId,

    /// Team the ant corresponds to
    pub team: u64,

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

    /// Damage the ant does
    pub damage: f32,

    /// Time to hatch from an egg
    pub hatch_time: f32,

    /// Current resource carry capacity
    pub carry: f32,

    /// Maximum resource carry capacity
    pub max_carry: f32,

    /// Whether the ant can fly
    pub can_fly: bool,

    /// Default behavior of the ant
    pub behavior: Behavior,

    /// Current action performed by the ant
    pub action: Action,

    /// Description of the ant
    pub description: String,
}

impl Default for AntCmp {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: Ant::BlackAnt,
            key: None,
            owner: 0,
            team: 0,
            scale: 0.03,
            z_score: 0.9,
            price: 0.,
            health: 0.,
            max_health: 0.,
            speed: DEFAULT_WALK_SPEED,
            damage: 0.,
            hatch_time: 0.,
            carry: 0.,
            max_carry: 1.,
            can_fly: false,
            behavior: Behavior::Attack,
            action: Action::Idle,
            description: "".to_string(),
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
                team: id,
                z_score: 0.1,
                price: 30.,
                health: 10.,
                max_health: 10.,
                damage: 2.,
                hatch_time: 5.,
                max_carry: 30.,
                behavior: Behavior::Harvest,
                action: Action::Idle,
                description: "\
                    The worker is the most common ant in the colony. \
                    They are responsible for gathering food."
                    .to_string(),
                ..default()
            },
            Ant::BlackBullet => Self {
                kind: Ant::BlackBullet,
                key: Some(KeyCode::KeyX),
                owner: id,
                team: id,
                z_score: 0.2,
                price: 100.,
                health: 10.,
                max_health: 10.,
                speed: DEFAULT_WALK_SPEED + 10.,
                damage: 3.,
                behavior: Behavior::Dig,
                action: Action::Idle,
                hatch_time: 10.,
                description: "\
                    The bullet ant expands the colonies territory digging \
                    new tunnels. They move fast, but are weak in combat."
                    .to_string(),
                ..default()
            },
            Ant::BlackSoldier => Self {
                kind: Ant::BlackSoldier,
                key: Some(KeyCode::KeyC),
                owner: id,
                team: id,
                scale: 0.04,
                z_score: 0.5,
                price: 150.,
                health: 50.,
                max_health: 50.,
                speed: DEFAULT_WALK_SPEED + 5.,
                damage: 6.,
                hatch_time: 15.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                description: "\
                    The soldier ants form the colony's base defense. Their main \
                    task is to protect the workers and queen from any foe."
                    .to_string(),
                ..default()
            },
            Ant::BlackQueen => Self {
                kind: Ant::BlackQueen,
                owner: id,
                team: id,
                scale: 0.06,
                price: f32::MAX,
                health: 1000.,
                max_health: 1000.,
                speed: DEFAULT_WALK_SPEED - 2.,
                damage: 20.,
                hatch_time: 30.,
                behavior: Behavior::Brood,
                action: Action::Idle,
                description: "\
                    The queen is the heart of the colony. She is responsible for \
                    laying eggs and keeping the colony alive. If the queen dies, \
                    you lose the game!"
                    .to_string(),
                ..default()
            },
            Ant::GoldTail => Self {
                kind: Ant::GoldTail,
                key: Some(KeyCode::KeyV),
                owner: id,
                team: id,
                scale: 0.04,
                z_score: 0.6,
                price: 200.,
                health: 50.,
                max_health: 50.,
                speed: DEFAULT_WALK_SPEED,
                damage: 9.,
                hatch_time: 12.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                description: "\
                    The gold tail ant is a rare species that is known for its \
                    golden tail. They are strong and fast, making them excellent \
                    hunters."
                    .to_string(),
                ..default()
            },
            Ant::TrapJaw => Self {
                kind: Ant::TrapJaw,
                key: Some(KeyCode::KeyB),
                owner: id,
                team: id,
                scale: 0.05,
                z_score: 0.7,
                price: 250.,
                health: 100.,
                max_health: 100.,
                speed: DEFAULT_WALK_SPEED - 5.,
                damage: 12.,
                hatch_time: 20.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                description: "\
                    The trap jaw ant is a rare species that is known for its \
                    powerful jaws. They are slow, but very strong individuals."
                    .to_string(),
                ..default()
            },
            Ant::BlackAlate => Self {
                kind: Ant::BlackAlate,
                key: Some(KeyCode::KeyN),
                owner: id,
                team: id,
                scale: 0.05,
                z_score: 0.9,
                price: 350.,
                health: 150.,
                max_health: 150.,
                speed: DEFAULT_WALK_SPEED,
                damage: 10.,
                hatch_time: 30.,
                can_fly: true,
                behavior: Behavior::Attack,
                action: Action::Idle,
                description: "\
                    The flying ant, also known as alate, are the male individuals \
                    in the colony. They are incredibly fast and powerful units."
                    .to_string(),
                ..default()
            },
            Ant::BlackScorpion => Self {
                kind: Ant::BlackScorpion,
                owner: id,
                team: rng().random_range(100..1000),
                scale: 0.05,
                health: 100.,
                max_health: 100.,
                speed: DEFAULT_WALK_SPEED - 5.,
                damage: 5.,
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
            Action::Harvest | Action::Heal | Action::Dig(_) => Animation::LookAround,
            Action::Brood(_) | Action::Idle => Animation::Idle,
            Action::TargetedWalk(_) => {
                if self.can_fly {
                    Animation::Fly
                } else {
                    Animation::Walk
                }
            }
            Action::Walk(_) => Animation::Walk,
        }
    }
}

#[derive(Component, Clone)]
pub struct Egg {
    /// Id of the egg
    pub id: Uuid,

    /// Player id of the egg's owner
    pub owner: ClientId,

    /// Team the egg corresponds to
    pub team: u64,

    /// Current health
    pub health: f32,

    /// Maximum health
    pub max_health: f32,

    /// Time to hatch
    pub timer: Timer,

    /// Type of ant in the egg
    pub ant: Ant,
}

impl Egg {
    pub fn size(&self) -> Vec2 {
        self.ant.size().as_vec2()
    }

    pub fn scaled_size(&self) -> Vec2 {
        let ant_c = AntCmp::new(&self.ant, self.owner);
        ant_c.scaled_size() * 0.5
    }
}
