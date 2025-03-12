use bevy::color::Color;

// Menu
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.65, 0.35);

// Camera
pub const MIN_ZOOM: f32 = 0.2;
pub const MAX_ZOOM: f32 = 1.;
pub const ZOOM_FACTOR: f32 = 1.1;
pub const LERP_FACTOR: f32 = 0.05;

// Game settings
pub const MAX_GAME_SPEED: f32 = 3.;
pub const GAME_SPEED_STEP: f32 = 0.5;
pub const TRAIT_TIMER: f32 = 10.;
pub const MAX_TRAITS: usize = 5;

// Z-scores
pub const TILE_Z_SCORE: f32 = 0.;
pub const EGG_Z_SCORE: f32 = 1.;
pub const ANT_Z_SCORE: f32 = 2.;
pub const MAX_Z_SCORE: f32 = 10.;

// Map
pub const VISION_RANGE: u32 = 3;
pub const MAX_TERRAFORM_POINTS: f32 = 100.;
pub const TILE_LEAF_CHANCE: f32 = 0.1;

// Ants
pub const BROODING_TIME: f32 = 2.5; // Seconds the queen needs to place an egg
pub const EGG_HEALTH_FACTOR: f32 = 0.25; // Fraction of health the egg has compared to the ant
pub const HATCH_SPEED_FACTOR: f32 = 2.; // Increase in egg hatching speed for the trait breeding
pub const DEATH_TIME: f32 = 5.; // Seconds a corpse remains on the map
pub const DEFAULT_WALK_SPEED: f32 = 20.; // Base walking speed
pub const DIG_SPEED: f32 = 50.; // Terraform points per ant per second
pub const TUNNEL_SPEED_FACTOR: f32 = 2.; // Dig speed increase for the trait tunneling
pub const HARVEST_SPEED: f32 = 5.; // Food harvesting per ant per second
pub const HARVEST_SPEED_FACTOR: f32 = 2.; // Harvesting speed increase for trait harvesting
pub const HARVEST_DECREASE_FACTOR: f32 = 0.5; // Harvesting speed decrease for trait warlike
pub const HEAL_SPEED_RATIO: f32 = 0.1; // Health ratio healed per second
pub const FLY_SPEED_FACTOR: f32 = 2.; // Times flying is faster than base
pub const HASTE_SPEED_FACTOR: f32 = 2.; // Walk speed increase for the trait haste
pub const SAME_TUNNEL_DIG_CHANCE: f32 = 0.95; // Chance of continuing digging in the same tunnel
