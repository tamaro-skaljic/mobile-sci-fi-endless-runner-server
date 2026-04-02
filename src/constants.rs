// Player creation

pub const INITIAL_GEMS: u64 = 25;
pub const INITIAL_COINS: u64 = 0;
pub const INITIAL_MAX_ENERGY: u8 = 60;

// Revives

pub const MAX_REVIVES_PER_PLAYTHROUGH: u8 = 5;

// Energy

pub const ENERGY_PER_PURCHASE_OR_AD: u8 = 15;
pub const ENERGY_CONSUMPTION_INTERVAL_MICROS: i64 = 29_280_000; // 29.28 seconds (1/3 of a level, which's length was decided by the length of the in-game soundtrack)
pub const ENERGY_REGENERATION_INTERVAL_MICROS: i64 = 180_000_000; // 3 minutes
pub const MAX_ENERGY: u8 = 60;

// Validation

pub const MIN_PLAYER_NAME_LENGTH: usize = 3;
pub const MAX_PLAYER_NAME_LENGTH: usize = 20;

// Gems

pub const GEMS_PER_AD: u64 = 10;
pub const GET_GEMS_AD_COOLDOWN_HOURS: u64 = 6;

// Daily rewards

pub const ONE_MINUTE_IN_MICROS: i64 = 60 * 1_000_000;

pub const ONE_DAY_IN_MICROS: i64 = 86_400 * 1_000_000;

pub const DAILY_REWARD_GEMS: [u64; 8] = [10, 30, 60, 100, 150, 250, 500, 15];

// Power-up max levels

pub const MAGNET_RANGE_UPGRADE_MAX_LEVEL: u8 = 10;
pub const MAGNET_DURATION_UPGRADE_MAX_LEVEL: u8 = 20;
pub const MAGNET_SPAWN_CHANCE_UPGRADE_MAX_LEVEL: u8 = 20;
pub const SHIELD_COLLISIONS_UPGRADE_MAX_LEVEL: u8 = 3;
pub const SHIELD_DURATION_UPGRADE_MAX_LEVEL: u8 = 20;
pub const SHIELD_SPAWN_CHANCE_UPGRADE_MAX_LEVEL: u8 = 20;

// Cosmetic prices

pub const PLAYER_SKIN_GEM_PRICE: u64 = 250;
pub const LEVEL_SKIN_GEM_PRICE: u64 = 100;
pub const PLAYER_MOVEMENT_TRAIL_GEM_PRICE: u64 = 250;
