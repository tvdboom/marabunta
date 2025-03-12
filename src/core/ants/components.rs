use crate::core::constants::DEFAULT_WALK_SPEED;
use crate::core::map::loc::Loc;
use crate::core::map::tile::Tile;
use crate::core::player::{AntColor, Player};
use crate::core::traits::Trait;
use crate::utils::NameFromEnum;
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
    Attack,
    Die,
    Fly,
    LookAround,
    Idle,
    Walk,
}

#[derive(EnumIter, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ant {
    Queen,
    Worker,
    Excavator,
    Soldier,
    Warrior,
    Alate,
    Mastodon,
    BlackScorpion,
    YellowScorpion,
    Wasp,
}

impl Ant {
    pub fn is_ant(&self) -> bool {
        match self {
            Ant::Queen
            | Ant::Worker
            | Ant::Excavator
            | Ant::Soldier
            | Ant::Warrior
            | Ant::Alate
            | Ant::Mastodon => true,
            _ => false,
        }
    }

    pub fn is_scorpion(&self) -> bool {
        match self {
            Ant::BlackScorpion | Ant::YellowScorpion => true,
            _ => false,
        }
    }

    pub fn colors(&self) -> Box<dyn Iterator<Item = AntCmp>> {
        match self {
            Ant::Mastodon | Ant::BlackScorpion | Ant::YellowScorpion | Ant::Wasp => {
                Box::new(std::iter::once(AntCmp::base(self)))
            }
            _ => Box::new(
                [
                    AntCmp::base(self).with_color(&AntColor::Black),
                    AntCmp::base(self).with_color(&AntColor::Red),
                ]
                .into_iter(),
            ),
        }
    }

    pub fn all_animations(&self) -> Vec<Animation> {
        let exclude_animations = match self {
            Ant::Alate => vec![],
            Ant::BlackScorpion | Ant::YellowScorpion => vec![Animation::Fly, Animation::LookAround],
            Ant::Wasp => vec![Animation::LookAround],
            _ => vec![Animation::Fly],
        };

        Animation::iter()
            .filter(|a| !exclude_animations.contains(a))
            .collect()
    }

    pub fn frames(&self, animation: &Animation) -> u32 {
        match self {
            Ant::BlackScorpion | Ant::YellowScorpion => match animation {
                Animation::Attack => 10,
                Animation::Die => 5,
                Animation::Fly => 10,
                Animation::Idle => 20,
                Animation::Walk => 16,
                _ => unreachable!(),
            },
            Ant::Wasp => match animation {
                Animation::Attack | Animation::Fly => 10,
                Animation::Die => 5,
                Animation::Idle => 12,
                Animation::Walk => 16,
                _ => unreachable!(),
            }
            _ => match animation {
                Animation::Attack | Animation::Walk => 8,
                Animation::Die => 10,
                Animation::Fly => 12,
                Animation::LookAround | Animation::Idle => 20,
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

    /// Color of the ant. None for monsters
    pub color: Option<AntColor>,

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
            kind: Ant::Worker,
            key: None,
            owner: 0,
            team: 0,
            color: None,
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
    pub fn new(kind: &Ant, player: &Player) -> Self {
        match kind {
            Ant::Worker => Self {
                kind: Ant::Worker,
                key: Some(KeyCode::KeyZ),
                owner: player.id,
                team: player.id,
                color: Some(player.color.clone()),
                scale: if player.has_trait(&Trait::Warlike) {
                    0.04
                } else {
                    0.03
                },
                z_score: 0.1,
                price: 30.,
                health: if player.has_trait(&Trait::Warlike) {
                    20.
                } else {
                    10.
                },
                max_health: if player.has_trait(&Trait::Warlike) {
                    20.
                } else {
                    10.
                },
                damage: if player.has_trait(&Trait::Warlike) {
                    4.
                } else {
                    2.
                },
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
            Ant::Excavator => Self {
                kind: Ant::Excavator,
                key: Some(KeyCode::KeyX),
                owner: player.id,
                team: player.id,
                color: Some(player.color.clone()),
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
                    The excavator ants expands the colonies territory digging \
                    new tunnels. They move fast, but are weak in combat."
                    .to_string(),
                ..default()
            },
            Ant::Soldier => Self {
                kind: Ant::Soldier,
                key: Some(KeyCode::KeyC),
                owner: player.id,
                team: player.id,
                color: Some(player.color.clone()),
                scale: if player.has_trait(&Trait::EnhancedSoldiers) {
                    0.05
                } else {
                    0.04
                },
                z_score: 0.5,
                price: 150.,
                health: 50.,
                max_health: 50.,
                speed: if player.has_trait(&Trait::EnhancedSoldiers) {
                    DEFAULT_WALK_SPEED + 10.
                } else {
                    DEFAULT_WALK_SPEED
                },
                damage: if player.has_trait(&Trait::EnhancedSoldiers) {
                    9.
                } else {
                    6.
                },
                hatch_time: 15.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                description: "\
                    The soldiers form the colony's base defense. Their main \
                    task is to protect the workers and queen from any foe."
                    .to_string(),
                ..default()
            },
            Ant::Queen => Self {
                kind: Ant::Queen,
                owner: player.id,
                team: player.id,
                color: Some(player.color.clone()),
                scale: if player.has_trait(&Trait::SuperQueen) {
                    0.08
                } else {
                    0.06
                },
                price: f32::MAX,
                health: if player.has_trait(&Trait::SuperQueen) {
                    1500.
                } else {
                    1000.
                },
                max_health: if player.has_trait(&Trait::SuperQueen) {
                    1500.
                } else {
                    1000.
                },
                speed: if player.has_trait(&Trait::SuperQueen) {
                    DEFAULT_WALK_SPEED - 6.
                } else {
                    DEFAULT_WALK_SPEED - 2.
                },
                damage: if player.has_trait(&Trait::SuperQueen) {
                    40.
                } else {
                    20.
                },
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
            Ant::Warrior => Self {
                kind: Ant::Warrior,
                key: Some(KeyCode::KeyV),
                owner: player.id,
                team: player.id,
                color: Some(player.color.clone()),
                scale: 0.04,
                z_score: 0.6,
                price: 200.,
                health: 50.,
                max_health: 50.,
                speed: DEFAULT_WALK_SPEED + 5.,
                damage: 9.,
                hatch_time: 12.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                description: "\
                    The warrior ants are the elite fighting units in the colony. \
                    They are stronger and faster than the soldier ants."
                    .to_string(),
                ..default()
            },
            Ant::Alate => Self {
                kind: Ant::Alate,
                key: Some(KeyCode::KeyN),
                owner: player.id,
                team: player.id,
                color: Some(player.color.clone()),
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
                    The alates, also known as flying ants, are the male individuals \
                    in the colony. They are incredibly fast and powerful units."
                    .to_string(),
                ..default()
            },
            Ant::Mastodon => Self {
                kind: Ant::Mastodon,
                key: Some(KeyCode::KeyB),
                owner: player.id,
                team: player.id,
                color: Some(player.color.clone()),
                scale: 0.06,
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
                    Mastodon ants are a rare kind that is known for its \
                    powerful jaws. They are slow, but very strong."
                    .to_string(),
                ..default()
            },
            Ant::BlackScorpion => Self {
                kind: Ant::BlackScorpion,
                owner: player.id,
                team: rng().random_range(100..10000),
                scale: 0.05,
                health: 100.,
                max_health: 100.,
                speed: DEFAULT_WALK_SPEED - 5.,
                damage: 5.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                ..default()
            },
            Ant::YellowScorpion => Self {
                kind: Ant::YellowScorpion,
                owner: player.id,
                team: rng().random_range(100..10000),
                scale: 0.05,
                health: 300.,
                max_health: 300.,
                speed: DEFAULT_WALK_SPEED,
                damage: 25.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                ..default()
            },
            Ant::Wasp => Self {
                kind: Ant::Wasp,
                owner: player.id,
                team: 90,
                scale: 0.05,
                health: 300.,
                max_health: 300.,
                speed: DEFAULT_WALK_SPEED,
                damage: 15.,
                behavior: Behavior::Attack,
                action: Action::Idle,
                ..default()
            },
        }
    }

    pub fn base(kind: &Ant) -> Self {
        Self::new(kind, &Player::default())
    }

    pub fn with_color(mut self, color: &AntColor) -> Self {
        self.color = Some(color.clone());
        self
    }

    pub fn folder(&self) -> String {
        if self.kind.colors().count() > 1 {
            if let Some(color) = &self.color {
                return format!("{}_{}", color.to_snake(), self.kind.to_snake());
            }
        }

        self.kind.to_snake()
    }

    pub fn atlas(&self, animation: &Animation) -> String {
        format!("{}_{}", self.folder(), animation.to_snake())
    }

    pub fn size(&self) -> Vec2 {
        match self.kind {
            Ant::Queen => Vec2::new(307., 525.),
            Ant::Worker => Vec2::new(307., 438.),
            Ant::Excavator => Vec2::new(307., 474.),
            Ant::Soldier => match self.color {
                Some(AntColor::Black) => Vec2::new(367., 508.),
                Some(AntColor::Red) => Vec2::new(361., 510.),
                _ => unreachable!(),
            },
            Ant::Warrior => match self.color {
                Some(AntColor::Black) => Vec2::new(466., 623.),
                Some(AntColor::Red) => Vec2::new(472., 560.),
                _ => unreachable!(),
            },
            Ant::Alate => Vec2::new(510., 512.),
            Ant::Mastodon => Vec2::new(513., 577.),
            Ant::BlackScorpion | Ant::YellowScorpion => Vec2::new(675., 785.),
            Ant::Wasp => Vec2::new(832., 676.),
        }
    }

    pub fn scaled_size(&self) -> Vec2 {
        self.size() * self.scale
    }

    pub fn animation(&self) -> Animation {
        match self.action {
            Action::Attack(_) => Animation::Attack,
            Action::Die(_) => Animation::Die,
            Action::Dig(_) | Action::Harvest => Animation::LookAround,
            Action::Heal => {
                if self.kind == Ant::Queen {
                    Animation::Idle
                } else {
                    Animation::LookAround
                }
            }
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

    /// Ant in the egg
    pub ant: AntCmp,
}

impl Egg {
    pub fn scaled_size(&self) -> Vec2 {
        self.ant.scaled_size() * 0.5
    }
}
