use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    constants::{
        MAGNET_DURATION_UPGRADE_MAX_LEVEL, MAGNET_RANGE_UPGRADE_MAX_LEVEL,
        MAGNET_SPAWN_CHANCE_UPGRADE_MAX_LEVEL, SHIELD_COLLISIONS_UPGRADE_MAX_LEVEL,
        SHIELD_DURATION_UPGRADE_MAX_LEVEL, SHIELD_SPAWN_CHANCE_UPGRADE_MAX_LEVEL,
    },
    purchase_price::{self, PricingMode},
    remove_from_wallet::charge_wallet,
    tables::{
        magnet_data::{GetMagnetDataRowOptionByPlayerId, UpdateMagnetDataRowByPlayerId},
        player::Player,
        purchase::Currency,
        shield_data::{GetShieldDataRowOptionByPlayerId, UpdateShieldDataRowByPlayerId},
    },
};

pub enum MagnetUpgrade {
    Range,
    Duration,
    SpawnChance,
}

pub enum ShieldUpgrade {
    Collisions,
    Duration,
    SpawnChance,
}

macro_rules! purchase_power_up_upgrade {
    (
        $fn_name:ident,
        $get:ident,
        $update:ident,
        $upgrade_type:ty,
        $label:expr,
        $($variant:ident => $get_level:ident, $set_level:ident, $max:expr, $price_fn:path),+ $(,)?
    ) => {
        pub fn $fn_name(
            dsl: &DSL<'_, ReducerContext>,
            player: &Player,
            currency: &Option<Currency>,
            upgrade: $upgrade_type,
        ) -> Result<(u64, u64), CheatOrError> {
            let mut data = dsl.$get(player)?;

            let (current_level, max_level, price_fn): (u8, u8, fn(u8) -> Option<PricingMode>) = match upgrade {
                $(
                    <$upgrade_type>::$variant => (
                        *data.$get_level(),
                        $max,
                        $price_fn,
                    ),
                )+
            };

            let price = match price_fn(current_level) {
                Some(price) => price,
                None => {
                    return Err(cheat_attempt(
                        dsl,
                        player,
                        &format!(
                            "Tried to purchase a {} upgrade beyond max level ({current_level}/{max_level})",
                            $label
                        ),
                    ));
                }
            };

            let charge = charge_wallet(dsl, player, &price, currency)?;

            match upgrade {
                $(
                    <$upgrade_type>::$variant => {
                        data.$set_level(current_level + 1);
                    }
                )+
            }
            dsl.$update(data)?;

            Ok(charge)
        }
    };
}

purchase_power_up_upgrade!(
    purchase_magnet_upgrade,
    get_magnet_data_by_player_id,
    update_magnet_data_by_player_id,
    MagnetUpgrade,
    "magnet",
    Range => get_range_upgrade_level, set_range_upgrade_level, MAGNET_RANGE_UPGRADE_MAX_LEVEL, purchase_price::magnet_range_upgrade_price,
    Duration => get_duration_upgrade_level, set_duration_upgrade_level, MAGNET_DURATION_UPGRADE_MAX_LEVEL, purchase_price::magnet_duration_upgrade_price,
    SpawnChance => get_spawn_chance_upgrade_level, set_spawn_chance_upgrade_level, MAGNET_SPAWN_CHANCE_UPGRADE_MAX_LEVEL, purchase_price::magnet_spawn_chance_upgrade_price,
);

purchase_power_up_upgrade!(
    purchase_shield_upgrade,
    get_shield_data_by_player_id,
    update_shield_data_by_player_id,
    ShieldUpgrade,
    "shield",
    Collisions => get_collisions_upgrade_level, set_collisions_upgrade_level, SHIELD_COLLISIONS_UPGRADE_MAX_LEVEL, purchase_price::shield_collisions_upgrade_price,
    Duration => get_duration_upgrade_level, set_duration_upgrade_level, SHIELD_DURATION_UPGRADE_MAX_LEVEL, purchase_price::shield_duration_upgrade_price,
    SpawnChance => get_spawn_chance_upgrade_level, set_spawn_chance_upgrade_level, SHIELD_SPAWN_CHANCE_UPGRADE_MAX_LEVEL, purchase_price::shield_spawn_chance_upgrade_price,
);
