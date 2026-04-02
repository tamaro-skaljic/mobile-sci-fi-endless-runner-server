use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    add_to_wallet::add_to_wallet,
    cheat_attempt_log::cheat_attempt,
    daily_reward_gems::daily_reward_gems,
    or_ok_on_cheat,
    tables::player::{GetPlayerRowOptionById, PlayerId, UpdatePlayerRowById},
    views::daily_reward_widgets::is_daily_reward_claimable,
};

#[reducer]
pub fn claim_daily_reward(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let sender = dsl.ctx().sender();
    let mut player = dsl.get_player_by_id(PlayerId::new(sender))?;

    if !is_daily_reward_claimable(&player, dsl.ctx().timestamp) {
        or_ok_on_cheat!(Err(cheat_attempt(
            &dsl,
            &player,
            "Tried to claim a daily reward, but already claimed one today",
        )));
    }

    // determine gem reward based on lifetime claim count
    let gems = daily_reward_gems(*player.get_number_of_claimed_daily_rewards());

    add_to_wallet(&dsl, &player, gems, 0)?;

    // update player claim tracking
    player.set_last_daily_reward_claimed_at(Some(dsl.ctx().timestamp));
    player.set_number_of_claimed_daily_rewards(player.get_number_of_claimed_daily_rewards() + 1);
    dsl.update_player_by_id(player)?;

    Ok(())
}
