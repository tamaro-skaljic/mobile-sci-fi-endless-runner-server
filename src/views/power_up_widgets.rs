use spacetimedb::{SpacetimeType, ViewContext};
use spacetimedsl::prelude::*;

use crate::{
    checks::is_same_local_day::is_same_local_day,
    purchase_price::{self, PricingMode},
    tables::{
        magnet_data::magnet_data__view, player::player__view, shield_data::shield_data__view,
    },
};

macro_rules! power_up_widget_view {
    (
        $view_name:ident, $widgets_struct:ident, $upgrade_price_enum:ident,
        $table_accessor:ident,
        $purchase_price_fn:path,
        $first_level_field:ident, $first_price_field:ident,
        $get_first_level:ident, $first_price_fn:path,
        $duration_price_fn:path, $spawn_chance_price_fn:path
    ) => {
        #[derive(SpacetimeType, Clone, Debug, PartialEq)]
        pub enum $upgrade_price_enum {
            Available(PricingMode),
            MaxLevel,
        }

        #[derive(SpacetimeType, Clone, Debug, PartialEq)]
        pub struct $widgets_struct {
            pub amount: u8,
            pub $first_level_field: u8,
            pub duration_upgrade_level: u8,
            pub spawn_chance_upgrade_level: u8,
            pub purchase_price: PricingMode,
            pub $first_price_field: $upgrade_price_enum,
            pub duration_upgrade_price: $upgrade_price_enum,
            pub spawn_chance_upgrade_price: $upgrade_price_enum,
        }

        impl $upgrade_price_enum {
            fn from_option(price: Option<PricingMode>) -> Self {
                match price {
                    Some(pricing_mode) => Self::Available(pricing_mode),
                    None => Self::MaxLevel,
                }
            }
        }

        #[spacetimedb::view(accessor = $view_name, public)]
        pub fn $view_name(ctx: &ViewContext) -> Option<$widgets_struct> {
            let player = ctx.db.player().id().find(ctx.sender())?;
            let data = ctx.db.$table_accessor().player_id().find(ctx.sender())?;
            let current_timestamp = ctx.timestamp().ok()?;

            let purchased_today = if is_same_local_day(
                *data.get_last_purchase_day(),
                current_timestamp,
                *player.get_time_difference_from_utc_in_minutes(),
            ) {
                *data.get_purchased_today()
            } else {
                0
            };

            Some($widgets_struct {
                amount: *data.get_amount(),
                $first_level_field: *data.$get_first_level(),
                duration_upgrade_level: *data.get_duration_upgrade_level(),
                spawn_chance_upgrade_level: *data.get_spawn_chance_upgrade_level(),
                purchase_price: $purchase_price_fn(purchased_today),
                $first_price_field: $upgrade_price_enum::from_option($first_price_fn(
                    *data.$get_first_level(),
                )),
                duration_upgrade_price: $upgrade_price_enum::from_option($duration_price_fn(
                    *data.get_duration_upgrade_level(),
                )),
                spawn_chance_upgrade_price: $upgrade_price_enum::from_option(
                    $spawn_chance_price_fn(*data.get_spawn_chance_upgrade_level()),
                ),
            })
        }
    };
}

power_up_widget_view!(
    magnet_widgets,
    MagnetWidgets,
    MagnetUpgradePrice,
    magnet_data,
    purchase_price::magnet_purchase_price,
    range_upgrade_level,
    range_upgrade_price,
    get_range_upgrade_level,
    purchase_price::magnet_range_upgrade_price,
    purchase_price::magnet_duration_upgrade_price,
    purchase_price::magnet_spawn_chance_upgrade_price
);

power_up_widget_view!(
    shield_widgets,
    ShieldWidgets,
    ShieldUpgradePrice,
    shield_data,
    purchase_price::shield_purchase_price,
    collisions_upgrade_level,
    collisions_upgrade_price,
    get_collisions_upgrade_level,
    purchase_price::shield_collisions_upgrade_price,
    purchase_price::shield_duration_upgrade_price,
    purchase_price::shield_spawn_chance_upgrade_price
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_price_available_when_below_max() {
        let price = MagnetUpgradePrice::from_option(Some(PricingMode::Coins(1000)));
        assert_eq!(
            price,
            MagnetUpgradePrice::Available(PricingMode::Coins(1000))
        );
    }

    #[test]
    fn upgrade_price_max_level_when_none() {
        let price = MagnetUpgradePrice::from_option(None);
        assert_eq!(price, MagnetUpgradePrice::MaxLevel);
    }

    #[test]
    fn shield_upgrade_price_available_when_below_max() {
        let price = ShieldUpgradePrice::from_option(Some(PricingMode::Gems(500)));
        assert_eq!(price, ShieldUpgradePrice::Available(PricingMode::Gems(500)));
    }

    #[test]
    fn shield_upgrade_price_max_level_when_none() {
        let price = ShieldUpgradePrice::from_option(None);
        assert_eq!(price, ShieldUpgradePrice::MaxLevel);
    }
}
