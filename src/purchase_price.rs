use spacetimedb::SpacetimeType;

use crate::constants::{
    LEVEL_SKIN_GEM_PRICE, PLAYER_MOVEMENT_TRAIL_GEM_PRICE, PLAYER_SKIN_GEM_PRICE,
};

const ENERGY_GEM_COST_BASE: u64 = 3;
const REVIVE_GEM_COST_BASE: u64 = 3;
const MAGNET_GEM_COST_PER_PURCHASE: u64 = 5;
const SHIELD_GEM_COST_PER_PURCHASE: u64 = 5;

const MAGNET_RANGE_UPGRADE_PRICES: [(u64, u64); 10] = [
    (0, 1_000),
    (100, 0),
    (0, 4_000),
    (400, 0),
    (0, 9_000),
    (900, 0),
    (0, 16_000),
    (1_600, 0),
    (0, 25_000),
    (2_500, 0),
];

const TWENTY_LEVEL_UPGRADE_PRICES: [(u64, u64); 20] = [
    (0, 250),
    (25, 0),
    (0, 750),
    (75, 0),
    (0, 1_500),
    (150, 0),
    (0, 2_500),
    (250, 0),
    (0, 3_750),
    (375, 0),
    (0, 5_250),
    (525, 0),
    (0, 7_000),
    (700, 0),
    (0, 9_000),
    (900, 0),
    (0, 11_250),
    (1_125, 0),
    (0, 13_750),
    (1_375, 0),
];

const SHIELD_COLLISIONS_UPGRADE_PRICES: [(u64, u64); 3] =
    [(0, 50_000), (0, 250_000), (0, 1_000_000)];

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct GemsAndCoinsPrice {
    pub gems: u64,
    pub coins: u64,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum PricingMode {
    Gems(u64),
    Coins(u64),
    GemsAndCoins(GemsAndCoinsPrice),
    GemsOrCoins(GemsAndCoinsPrice),
}

impl PricingMode {
    pub fn gems(&self) -> u64 {
        match self {
            PricingMode::Gems(gems) => *gems,
            PricingMode::Coins(_) => 0,
            PricingMode::GemsAndCoins(price) => price.gems,
            PricingMode::GemsOrCoins(price) => price.gems,
        }
    }

    pub fn coins(&self) -> u64 {
        match self {
            PricingMode::Gems(_) => 0,
            PricingMode::Coins(coins) => *coins,
            PricingMode::GemsAndCoins(price) => price.coins,
            PricingMode::GemsOrCoins(price) => price.coins,
        }
    }
}

fn price_from_table(table: &[(u64, u64)], current_level: u8) -> Option<PricingMode> {
    let index = current_level as usize;
    let (gems, coins) = *table.get(index)?;
    Some(match (gems, coins) {
        (0, coins) => PricingMode::Coins(coins),
        (gems, 0) => PricingMode::Gems(gems),
        (gems, coins) => PricingMode::GemsAndCoins(GemsAndCoinsPrice { gems, coins }),
    })
}

pub fn energy_purchase_price(energy_purchases: u8) -> PricingMode {
    PricingMode::Gems(ENERGY_GEM_COST_BASE + (ENERGY_GEM_COST_BASE * energy_purchases as u64))
}

pub fn energy_purchase_price_without_playthrough() -> PricingMode {
    PricingMode::Gems(ENERGY_GEM_COST_BASE)
}

pub fn revive_purchase_price(revive_count: u8) -> PricingMode {
    PricingMode::Gems(REVIVE_GEM_COST_BASE + (REVIVE_GEM_COST_BASE * revive_count as u64))
}

pub fn magnet_purchase_price(purchased_today: u8) -> PricingMode {
    PricingMode::Gems(MAGNET_GEM_COST_PER_PURCHASE * purchased_today as u64)
}

pub fn shield_purchase_price(purchased_today: u8) -> PricingMode {
    PricingMode::Gems(SHIELD_GEM_COST_PER_PURCHASE * purchased_today as u64)
}

pub fn magnet_range_upgrade_price(current_level: u8) -> Option<PricingMode> {
    price_from_table(&MAGNET_RANGE_UPGRADE_PRICES, current_level)
}

pub fn magnet_duration_upgrade_price(current_level: u8) -> Option<PricingMode> {
    price_from_table(&TWENTY_LEVEL_UPGRADE_PRICES, current_level)
}

pub fn magnet_spawn_chance_upgrade_price(current_level: u8) -> Option<PricingMode> {
    price_from_table(&TWENTY_LEVEL_UPGRADE_PRICES, current_level)
}

pub fn shield_collisions_upgrade_price(current_level: u8) -> Option<PricingMode> {
    price_from_table(&SHIELD_COLLISIONS_UPGRADE_PRICES, current_level)
}

pub fn shield_duration_upgrade_price(current_level: u8) -> Option<PricingMode> {
    price_from_table(&TWENTY_LEVEL_UPGRADE_PRICES, current_level)
}

pub fn shield_spawn_chance_upgrade_price(current_level: u8) -> Option<PricingMode> {
    price_from_table(&TWENTY_LEVEL_UPGRADE_PRICES, current_level)
}

pub fn player_skin_price() -> PricingMode {
    PricingMode::Gems(PLAYER_SKIN_GEM_PRICE)
}

pub fn level_skin_price() -> PricingMode {
    PricingMode::Gems(LEVEL_SKIN_GEM_PRICE)
}

pub fn player_movement_trail_price() -> PricingMode {
    PricingMode::Gems(PLAYER_MOVEMENT_TRAIL_GEM_PRICE)
}

pub fn is_revive_ad_available(price: &PricingMode) -> bool {
    price.gems() <= REVIVE_GEM_COST_BASE * 3
}

pub fn is_energy_ad_available(price: &PricingMode) -> bool {
    price.gems() <= ENERGY_GEM_COST_BASE * 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        MAGNET_DURATION_UPGRADE_MAX_LEVEL, MAGNET_RANGE_UPGRADE_MAX_LEVEL,
        MAGNET_SPAWN_CHANCE_UPGRADE_MAX_LEVEL, SHIELD_COLLISIONS_UPGRADE_MAX_LEVEL,
        SHIELD_DURATION_UPGRADE_MAX_LEVEL, SHIELD_SPAWN_CHANCE_UPGRADE_MAX_LEVEL,
    };

    #[test]
    fn energy_purchase_price_first_purchase_during_playthrough() {
        let price = energy_purchase_price(0);
        assert_eq!(price.gems(), 3);
        assert_eq!(price.coins(), 0);
        assert_eq!(price, PricingMode::Gems(3));
    }

    #[test]
    fn energy_purchase_price_escalates_with_purchases() {
        assert_eq!(energy_purchase_price(0).gems(), 3);
        assert_eq!(energy_purchase_price(1).gems(), 6);
        assert_eq!(energy_purchase_price(2).gems(), 9);
        assert_eq!(energy_purchase_price(3).gems(), 12);
    }

    #[test]
    fn energy_purchase_price_without_playthrough_is_flat() {
        let price = energy_purchase_price_without_playthrough();
        assert_eq!(price.gems(), 3);
        assert_eq!(price.coins(), 0);
    }

    #[test]
    fn revive_purchase_price_first_revive() {
        let price = revive_purchase_price(0);
        assert_eq!(price.gems(), 3);
        assert_eq!(price.coins(), 0);
    }

    #[test]
    fn revive_purchase_price_escalates_with_revives() {
        assert_eq!(revive_purchase_price(0).gems(), 3);
        assert_eq!(revive_purchase_price(1).gems(), 6);
        assert_eq!(revive_purchase_price(2).gems(), 9);
        assert_eq!(revive_purchase_price(3).gems(), 12);
        assert_eq!(revive_purchase_price(4).gems(), 15);
    }

    #[test]
    fn magnet_purchase_price_first_of_day_is_free() {
        let price = magnet_purchase_price(0);
        assert_eq!(price.gems(), 0);
        assert_eq!(price.coins(), 0);
    }

    #[test]
    fn magnet_purchase_price_escalates_per_day() {
        assert_eq!(magnet_purchase_price(0).gems(), 0);
        assert_eq!(magnet_purchase_price(1).gems(), 5);
        assert_eq!(magnet_purchase_price(2).gems(), 10);
        assert_eq!(magnet_purchase_price(3).gems(), 15);
    }

    #[test]
    fn shield_purchase_price_first_of_day_is_free() {
        let price = shield_purchase_price(0);
        assert_eq!(price.gems(), 0);
        assert_eq!(price.coins(), 0);
    }

    #[test]
    fn shield_purchase_price_escalates_per_day() {
        assert_eq!(shield_purchase_price(0).gems(), 0);
        assert_eq!(shield_purchase_price(1).gems(), 5);
        assert_eq!(shield_purchase_price(2).gems(), 10);
        assert_eq!(shield_purchase_price(3).gems(), 15);
    }

    #[test]
    fn magnet_range_upgrade_price_all_levels() {
        let expected = [
            (0, 1_000),
            (100, 0),
            (0, 4_000),
            (400, 0),
            (0, 9_000),
            (900, 0),
            (0, 16_000),
            (1_600, 0),
            (0, 25_000),
            (2_500, 0),
        ];
        for (level, (gems, coins)) in expected.iter().enumerate() {
            let price = magnet_range_upgrade_price(level as u8).unwrap();
            assert_eq!(price.gems(), *gems, "level {level} gems");
            assert_eq!(price.coins(), *coins, "level {level} coins");
        }
    }

    #[test]
    fn magnet_range_upgrade_price_returns_none_at_max() {
        assert!(magnet_range_upgrade_price(MAGNET_RANGE_UPGRADE_MAX_LEVEL).is_none());
    }

    #[test]
    fn twenty_level_upgrade_price_all_levels() {
        let expected = [
            (0, 250),
            (25, 0),
            (0, 750),
            (75, 0),
            (0, 1_500),
            (150, 0),
            (0, 2_500),
            (250, 0),
            (0, 3_750),
            (375, 0),
            (0, 5_250),
            (525, 0),
            (0, 7_000),
            (700, 0),
            (0, 9_000),
            (900, 0),
            (0, 11_250),
            (1_125, 0),
            (0, 13_750),
            (1_375, 0),
        ];
        for (level, (gems, coins)) in expected.iter().enumerate() {
            let price = magnet_duration_upgrade_price(level as u8).unwrap();
            assert_eq!(price.gems(), *gems, "level {level} gems");
            assert_eq!(price.coins(), *coins, "level {level} coins");
        }
    }

    #[test]
    fn twenty_level_upgrade_returns_none_at_max() {
        assert!(magnet_duration_upgrade_price(MAGNET_DURATION_UPGRADE_MAX_LEVEL).is_none());
        assert!(magnet_spawn_chance_upgrade_price(MAGNET_SPAWN_CHANCE_UPGRADE_MAX_LEVEL).is_none());
        assert!(shield_duration_upgrade_price(SHIELD_DURATION_UPGRADE_MAX_LEVEL).is_none());
        assert!(shield_spawn_chance_upgrade_price(SHIELD_SPAWN_CHANCE_UPGRADE_MAX_LEVEL).is_none());
    }

    #[test]
    fn shield_collisions_upgrade_price_all_levels() {
        let expected = [(0, 50_000), (0, 250_000), (0, 1_000_000)];
        for (level, (gems, coins)) in expected.iter().enumerate() {
            let price = shield_collisions_upgrade_price(level as u8).unwrap();
            assert_eq!(price.gems(), *gems, "level {level} gems");
            assert_eq!(price.coins(), *coins, "level {level} coins");
        }
    }

    #[test]
    fn shield_collisions_upgrade_price_returns_none_at_max() {
        assert!(shield_collisions_upgrade_price(SHIELD_COLLISIONS_UPGRADE_MAX_LEVEL).is_none());
    }

    #[test]
    fn cosmetic_prices() {
        let skin = player_skin_price();
        assert_eq!(skin.gems(), 250);
        assert_eq!(skin.coins(), 0);

        let level = level_skin_price();
        assert_eq!(level.gems(), 100);
        assert_eq!(level.coins(), 0);

        let trail = player_movement_trail_price();
        assert_eq!(trail.gems(), 250);
        assert_eq!(trail.coins(), 0);
    }

    #[test]
    fn revive_ad_available_for_first_three() {
        assert!(is_revive_ad_available(&revive_purchase_price(0)));
        assert!(is_revive_ad_available(&revive_purchase_price(1)));
        assert!(is_revive_ad_available(&revive_purchase_price(2)));
        assert!(!is_revive_ad_available(&revive_purchase_price(3)));
        assert!(!is_revive_ad_available(&revive_purchase_price(4)));
    }

    #[test]
    fn energy_ad_available_for_first_three() {
        assert!(is_energy_ad_available(&energy_purchase_price(0)));
        assert!(is_energy_ad_available(&energy_purchase_price(1)));
        assert!(is_energy_ad_available(&energy_purchase_price(2)));
        assert!(!is_energy_ad_available(&energy_purchase_price(3)));
        assert!(!is_energy_ad_available(&energy_purchase_price(4)));
    }

    #[test]
    fn energy_ad_always_available_outside_playthrough() {
        assert!(is_energy_ad_available(
            &energy_purchase_price_without_playthrough()
        ));
    }

    #[test]
    fn all_functions_sharing_twenty_level_table_return_same_prices() {
        for level in 0..20u8 {
            let magnet_duration = magnet_duration_upgrade_price(level).unwrap();
            let magnet_spawn = magnet_spawn_chance_upgrade_price(level).unwrap();
            let shield_duration = shield_duration_upgrade_price(level).unwrap();
            let shield_spawn = shield_spawn_chance_upgrade_price(level).unwrap();
            assert_eq!(magnet_duration, magnet_spawn, "level {level}");
            assert_eq!(magnet_duration, shield_duration, "level {level}");
            assert_eq!(magnet_duration, shield_spawn, "level {level}");
        }
    }
}
