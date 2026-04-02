use spacetimedb::{SpacetimeType, ViewContext};
use spacetimedsl::prelude::*;

use crate::{
    constants::MAX_REVIVES_PER_PLAYTHROUGH,
    energy::calculation::recalculate_energy_for_idle,
    purchase_price::{self, PricingMode},
    tables::{
        energy::energy__view,
        player::{Player, player__view},
        playthrough::{PauseReason, playthrough__view},
    },
};

macro_rules! panel_view {
    ($view_name:ident, $struct_name:ident, $price_fn:path, $ad_available_fn:path) => {
        #[derive(SpacetimeType, Clone, Debug, PartialEq)]
        pub struct $struct_name {
            pub price: PricingMode,
            pub ad_available: bool,
        }

        #[spacetimedb::view(accessor = $view_name, public)]
        pub fn $view_name(ctx: &ViewContext) -> Option<$struct_name> {
            let player = ctx.db.player().id().find(ctx.sender())?;
            let price = $price_fn(ctx, &player)?;
            let ad_available = $ad_available_fn(&price);
            Some($struct_name {
                price,
                ad_available,
            })
        }
    };
}

panel_view!(
    revive_panel,
    RevivePanel,
    revive_panel_price,
    purchase_price::is_revive_ad_available
);

panel_view!(
    energy_panel,
    EnergyPanel,
    energy_panel_price,
    purchase_price::is_energy_ad_available
);

fn revive_panel_price(ctx: &ViewContext, player: &Player) -> Option<PricingMode> {
    let playthrough_id = player.get_last_playthrough_id()?;
    let playthrough = ctx.db.playthrough().id().find(playthrough_id.value())?;

    if playthrough.get_end().is_some() {
        return None;
    }

    let last_pause = playthrough.get_pauses().last()?;
    if last_pause.end.is_some() || last_pause.reason != PauseReason::Revive {
        return None;
    }

    let revive_count = *playthrough.get_revive_count();
    if revive_count >= MAX_REVIVES_PER_PLAYTHROUGH {
        return None;
    }

    Some(purchase_price::revive_purchase_price(revive_count))
}

fn energy_panel_price(ctx: &ViewContext, player: &Player) -> Option<PricingMode> {
    match player.get_last_playthrough_id() {
        Some(playthrough_id) => {
            let playthrough = ctx.db.playthrough().id().find(playthrough_id.value())?;

            if playthrough.get_end().is_some() {
                return energy_panel_price_for_idle(ctx);
            }

            let last_pause = playthrough.get_pauses().last()?;
            if last_pause.end.is_some() || last_pause.reason != PauseReason::OutOfEnergy {
                return None;
            }

            Some(purchase_price::energy_purchase_price(
                *playthrough.get_energy_purchases(),
            ))
        }
        None => energy_panel_price_for_idle(ctx),
    }
}

fn energy_panel_price_for_idle(ctx: &ViewContext) -> Option<PricingMode> {
    let mut energy = ctx.db.energy().player_id().find(ctx.sender())?;
    let now = ctx.timestamp().ok()?;
    recalculate_energy_for_idle(&mut energy, now);

    if *energy.get_energy() > 1 {
        return None;
    }

    Some(purchase_price::energy_purchase_price_without_playthrough())
}
