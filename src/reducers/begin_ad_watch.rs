use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::cheat_attempt,
    checks::{
        playthrough_is_in_pause::playthrough_is_in_pause_reason,
        revive_is_allowed::revive_is_allowed,
    },
    or_ok_on_cheat, purchase_price,
    tables::{
        advertisement::{
            AdType, AdWatchStatus, CreateAdvertisementWatch, CreateAdvertisementWatchRow,
        },
        player::{GetPlayerRowOptionById, PlayerId},
        playthrough::{GetPlaythroughRowOptionById, PauseReason},
    },
};

#[reducer]
pub fn begin_ad_watch(ctx: &ReducerContext, ad_type: AdType) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let sender = dsl.ctx().sender();
    let player_id = PlayerId::new(sender);

    let instead_of = match ad_type {
        AdType::Gems | AdType::DoubleCoins => Some(0),
        AdType::Energy => {
            let player = dsl.get_player_by_id(&player_id)?;
            let energy_purchases = if let Some(playthrough_id) = player.get_last_playthrough_id() {
                let playthrough = dsl.get_playthrough_by_id(playthrough_id)?;
                if playthrough.get_end().is_none() {
                    or_ok_on_cheat!(playthrough_is_in_pause_reason(
                        &dsl,
                        &player,
                        &playthrough,
                        &PauseReason::OutOfEnergy,
                    ));
                    *playthrough.get_energy_purchases()
                } else {
                    0
                }
            } else {
                0
            };
            let price = purchase_price::energy_purchase_price(energy_purchases);
            if !purchase_price::is_energy_ad_available(&price) {
                or_ok_on_cheat!(Err(cheat_attempt(
                    &dsl,
                    &player,
                    "Energy ad not available at this purchase count",
                )));
            }
            Some(purchase_price::energy_purchase_price_without_playthrough().gems() as u32)
        }
        AdType::Revive => {
            let (_active_playthrough, revive_count) =
                or_ok_on_cheat!(revive_is_allowed(&dsl, player_id.clone()));
            let price = purchase_price::revive_purchase_price(revive_count);
            if !purchase_price::is_revive_ad_available(&price) {
                let player = dsl.get_player_by_id(&player_id)?;
                or_ok_on_cheat!(Err(cheat_attempt(
                    &dsl,
                    &player,
                    "Revive ad not available at this revive count",
                )));
            }
            Some(price.gems() as u32)
        }
    };

    dsl.create_advertisement_watch(CreateAdvertisementWatch {
        player_id,
        status: AdWatchStatus::Watching,
        instead_of,
        ad_type,
    })?;

    Ok(())
}
