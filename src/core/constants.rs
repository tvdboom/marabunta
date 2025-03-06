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

// Z-scores
pub const TILE_Z_SCORE: f32 = 0.;
pub const EGG_Z_SCORE: f32 = 1.;
pub const ANT_Z_SCORE: f32 = 2.;
pub const MAX_Z_SCORE: f32 = 10.;

// Map
pub const VISION_RANGE: u32 = 3;
pub const MAX_TERRAFORM_POINTS: f32 = 100.;
pub const TILE_LEAF_CHANCE: f32 = 0.1;
pub const TILE_ENEMY_CHANCE: f32 = 0.5;

// Ants
pub const BROODING_TIME: f32 = 0.001; //2.5; // Seconds the queen needs to place an egg
pub const DEATH_TIME: f32 = 5.; // Seconds a corpse remains on the map
pub const DEFAULT_WALK_SPEED: f32 = 20.; // Base walking speed
pub const DIG_SPEED: f32 = 50.; // Terraform points per ant per second
pub const HARVEST_SPEED: f32 = 10.; // Food harvesting per ant per second
pub const SAME_TUNNEL_DIG_CHANCE: f32 = 0.95; // Chance of continuing digging in the same tunnel
pub const ATTACK_CLOSEST_ENEMY_CHANCE: f32 = 0.8; // Chance of attacking the closest enemy
