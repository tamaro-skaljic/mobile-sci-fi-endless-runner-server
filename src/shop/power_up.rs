use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::CheatOrError,
    checks::is_same_local_day::is_same_local_day,
    purchase_price,
    remove_from_wallet::charge_wallet,
    tables::{
        magnet_data::{GetMagnetDataRowOptionByPlayerId, UpdateMagnetDataRowByPlayerId},
        player::Player,
        purchase::Currency,
        shield_data::{GetShieldDataRowOptionByPlayerId, UpdateShieldDataRowByPlayerId},
    },
};

macro_rules! purchase_power_up {
    (
        $fn_name:ident,
        $get:ident,
        $update:ident,
        $price_fn:path
    ) => {
        pub fn $fn_name(
            dsl: &DSL<'_, ReducerContext>,
            player: &Player,
            currency: &Option<Currency>,
        ) -> Result<(u64, u64), CheatOrError> {
            let mut data = dsl.$get(player)?;

            let purchased_today = if is_same_local_day(
                *data.get_last_purchase_day(),
                dsl.ctx().timestamp,
                *player.get_time_difference_from_utc_in_minutes(),
            ) {
                *data.get_purchased_today()
            } else {
                0
            };

            let price = $price_fn(purchased_today);
            let charge = charge_wallet(dsl, player, &price, currency)?;

            data.set_amount(*data.get_amount() + 1);
            data.set_purchased_today(purchased_today + 1);
            data.set_last_purchase_day(dsl.ctx().timestamp);
            dsl.$update(data)?;

            Ok(charge)
        }
    };
}

purchase_power_up!(
    purchase_magnet,
    get_magnet_data_by_player_id,
    update_magnet_data_by_player_id,
    purchase_price::magnet_purchase_price
);

purchase_power_up!(
    purchase_shield,
    get_shield_data_by_player_id,
    update_shield_data_by_player_id,
    purchase_price::shield_purchase_price
);
