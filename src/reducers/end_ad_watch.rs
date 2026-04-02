use spacetimedb::{ReducerContext, TimeDuration, reducer};
use spacetimedsl::prelude::*;

use crate::{
    add_to_wallet::add_to_wallet,
    authenticated_player::get_authenticated_player,
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    checks::{
        non_ended_playthrough::get_non_ended_playthrough,
        player_has_playthrough::player_has_playthrough,
        playthrough_is_active::playthrough_is_active,
        playthrough_is_in_pause::playthrough_is_in_pause_reason,
        revive_is_allowed::revive_is_allowed,
    },
    constants::{GEMS_PER_AD, GET_GEMS_AD_COOLDOWN_HOURS},
    energy::add_energy,
    or_ok_on_cheat, purchase_price,
    revive::revive,
    tables::{
        advertisement::{
            AdType, AdWatchStatus, AdvertisementWatchId, GetAdvertisementWatchRowOptionById,
            UpdateAdvertisementWatchRowById,
        },
        player::{Player, UpdatePlayerRowById},
        playthrough::{GetPlaythroughRowOptionById, PauseReason, UpdatePlaythroughRowById},
        revive::ReviveType,
    },
};

const GET_GEMS_AD_COOLDOWN_MICROS: i64 = GET_GEMS_AD_COOLDOWN_HOURS as i64 * 60 * 60 * 1_000_000;

#[reducer]
pub fn end_ad_watch(
    ctx: &ReducerContext,
    ad_watch_id: u64,
    ad_watch_status: AdWatchStatus,
) -> Result<(), SpacetimeDSLError> {
    let (dsl, player) = get_authenticated_player(ctx)?;

    let mut ad_watch =
        match dsl.get_advertisement_watch_by_id(AdvertisementWatchId::new(ad_watch_id)) {
            Ok(ad_watch) => ad_watch,
            Err(SpacetimeDSLError::NotFoundError { .. }) => {
                or_ok_on_cheat!(Err(cheat_attempt(
                    &dsl,
                    &player,
                    "Tried to end a non-existent ad watch",
                )))
            }
            Err(error) => return Err(error),
        };

    if *ad_watch.get_status() != AdWatchStatus::Watching {
        or_ok_on_cheat!(Err(cheat_attempt(
            &dsl,
            &player,
            "Attempted to end an ad watch that already ended",
        )));
    }

    if ad_watch.get_player_id() != player.get_id() {
        or_ok_on_cheat!(Err(cheat_attempt(
            &dsl,
            &player,
            "Attempted to end another player's ad watch",
        )));
    }

    ad_watch.set_status(ad_watch_status.clone());
    ad_watch = dsl.update_advertisement_watch_by_id(ad_watch)?;

    match ad_watch.get_status() {
        AdWatchStatus::Watching => {
            return Err(SpacetimeDSLError::Error(
                "Internal Server Error: Ad watch is still in progress".to_string(),
            ));
        }
        AdWatchStatus::Cancelled => return Ok(()),
        AdWatchStatus::Finished => match ad_watch.get_ad_type() {
            AdType::Energy => {
                let active_playthrough =
                    or_ok_on_cheat!(get_non_ended_playthrough_for_energy(&dsl, &player));
                let energy_purchases = match &active_playthrough {
                    Some(playthrough) => *playthrough.get_energy_purchases(),
                    None => 0,
                };
                let price = purchase_price::energy_purchase_price(energy_purchases);
                if !purchase_price::is_energy_ad_available(&price) {
                    or_ok_on_cheat!(Err(cheat_attempt(
                        &dsl,
                        &player,
                        "Energy ad not available at this purchase count",
                    )));
                }
                or_ok_on_cheat!(add_energy(&dsl, &player, &active_playthrough));
            }
            AdType::Revive => {
                let (_, revive_count) = or_ok_on_cheat!(revive_is_allowed(&dsl, player.get_id()));
                let price = purchase_price::revive_purchase_price(revive_count);
                if !purchase_price::is_revive_ad_available(&price) {
                    or_ok_on_cheat!(Err(cheat_attempt(
                        &dsl,
                        &player,
                        "Revive ad not available at this revive count",
                    )));
                }
                or_ok_on_cheat!(revive(&dsl, &player, ReviveType::AdWatch));
            }
            AdType::Gems => {
                reward_gems_ad_watch(&dsl, player)?;
            }
            AdType::DoubleCoins => {
                or_ok_on_cheat!(reward_double_coins_ad_watch(&dsl, &player));
            }
        },
    }

    Ok(())
}

fn get_non_ended_playthrough_for_energy(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
) -> Result<Option<crate::tables::playthrough::Playthrough>, CheatOrError> {
    if let Some(playthrough) = get_non_ended_playthrough(dsl, player) {
        playthrough_is_in_pause_reason(dsl, player, &playthrough, &PauseReason::OutOfEnergy)?;
        return Ok(Some(playthrough));
    }
    Ok(None)
}

fn reward_gems_ad_watch(
    dsl: &DSL<'_, ReducerContext>,
    mut player: Player,
) -> Result<(), SpacetimeDSLError> {
    add_to_wallet(dsl, &player, GEMS_PER_AD, 0)?;

    player.set_next_get_gems_ad_watch_available_at(
        dsl.ctx().timestamp + TimeDuration::from_micros(GET_GEMS_AD_COOLDOWN_MICROS),
    );
    dsl.update_player_by_id(player)?;

    Ok(())
}

fn reward_double_coins_ad_watch(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
) -> Result<(), CheatOrError> {
    let last_playthrough_id = player_has_playthrough(dsl, player)?;
    let mut active_playthrough = dsl.get_playthrough_by_id(last_playthrough_id)?;
    playthrough_is_active(dsl, player, &active_playthrough)?;

    if *active_playthrough.get_coins_doubled() {
        return Err(cheat_attempt(dsl, player, "Coins already doubled"));
    }

    let coins = *active_playthrough.get_coins();

    active_playthrough.set_coins_doubled(true);
    active_playthrough.set_coins(coins * 2);

    dsl.update_playthrough_by_id(active_playthrough)?;

    add_to_wallet(dsl, player, 0, coins)?;

    Ok(())
}
