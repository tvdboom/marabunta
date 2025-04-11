use bevy::color::Color;

// Menu
pub const LABEL_TEXT_SIZE: f32 = 10.;
pub const BUTTON_TEXT_SIZE: f32 = 20.;
pub const SUBTITLE_TEXT_SIZE: f32 = 15.;
pub const TITLE_TEXT_SIZE: f32 = 25.;
pub const NORMAL_BUTTON_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON_COLOR: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON_COLOR: Color = Color::srgb(0.35, 0.65, 0.35);
pub const DISABLED_BUTTON_COLOR: Color = Color::srgb(0.8, 0.5, 0.5);

// Camera
pub const MIN_ZOOM: f32 = 0.2;
pub const MAX_ZOOM: f32 = 1.;
pub const ZOOM_FACTOR: f32 = 1.1;
pub const LERP_FACTOR: f32 = 0.05;

// Game settings
pub const MAX_GAME_SPEED: f32 = 5.;
pub const GAME_SPEED_STEP: f32 = 0.5;
pub const MAX_QUEUE_LENGTH: usize = 12;
pub const TRAIT_TIMER: f32 = 120.;
pub const MAX_TRAITS: usize = 7;
pub const ENEMY_TIMER: u64 = 300;
pub const NETWORK_TIMER: u64 = 50;

// Z-scores
pub const TILE_Z_SCORE: f32 = 0.;
pub const EGG_Z_SCORE: f32 = 1.;
pub const ANT_Z_SCORE: f32 = 2.;
pub const MAX_Z_SCORE: f32 = 10.;

// Teams
pub const LEAF_TEAM: u64 = 50;
pub const TERMITE_TEAM: u64 = 80;
pub const WASP_TEAM: u64 = 90;

// Map
pub const NON_MAP_ID: u32 = 9999;
pub const VISION_RANGE: u32 = 2;
pub const MAX_TERRAFORM_POINTS: f32 = 100.;
pub const TILE_LEAF_CHANCE: f32 = 0.2;
pub const CAPPED_DELTA_SECS_SPEED: f32 = 0.05;

// Ants
pub const MONSTER_SPAWN_CHANCE: f32 = 0.015; // Chance of spawning wasps/termites every ENEMY_TIMER tick
pub const ANT_PRICE_FACTOR: f32 = 0.9; // Price reduction for trait megacolony
pub const BROODING_TIME: f32 = 2.5; // Seconds the queen needs to place an egg
pub const EGG_HEALTH_FACTOR: f32 = 0.25; // Fraction of health the egg has compared to the ant
pub const HATCH_SPEED_FACTOR: f32 = 2.; // Increase in egg hatching speed for the trait breeding
pub const DEATH_TIME: f32 = 15.; // Seconds a corpse remains on the map
pub const DEFAULT_WALK_SPEED: f32 = 20.; // Base walking speed
pub const DIG_SPEED: f32 = 10.; // Terraform points per ant per second
pub const TUNNEL_SPEED_FACTOR: f32 = 2.; // Dig speed increase for the trait tunneling
pub const HARVEST_SPEED: f32 = 5.; // Food harvesting per ant per second
pub const HARVEST_SPEED_FACTOR: f32 = 2.; // Harvesting speed increase for trait harvesting
pub const HARVEST_DECREASE_FACTOR: f32 = 0.5; // Harvesting speed decrease for trait warlike
pub const HEAL_SPEED_RATIO: f32 = 0.05; // Health ratio healed per second
pub const FLY_SPEED_FACTOR: f32 = 2.; // Times flying is faster than base
pub const HASTE_SPEED_FACTOR: f32 = 1.2; // Walk speed increase for the trait haste
pub const SAME_TUNNEL_DIG_CHANCE: f32 = 0.95; // Chance of continuing digging in the same tunnel
pub const MAX_DISTANCE_PROTECT: usize = 5; // Maximum distance of target to protect
