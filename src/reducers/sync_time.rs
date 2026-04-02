use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::cheat_attempt,
    or_ok_on_cheat,
    tables::player::{GetPlayerRowOptionById, PlayerId, UpdatePlayerRowById},
};

const MIN_TIME_DIFFERENCE_FROM_UTC_IN_MINUTES: i16 = -720;
const MAX_TIME_DIFFERENCE_FROM_UTC_IN_MINUTES: i16 = 840;

#[reducer]
pub fn sync_time(
    ctx: &ReducerContext,
    time_difference_from_utc_in_minutes: i16,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let mut player = dsl.get_player_by_id(PlayerId::new(dsl.ctx().sender()))?;

    if !(MIN_TIME_DIFFERENCE_FROM_UTC_IN_MINUTES..=MAX_TIME_DIFFERENCE_FROM_UTC_IN_MINUTES)
        .contains(&time_difference_from_utc_in_minutes)
    {
        or_ok_on_cheat!(Err(cheat_attempt(
            &dsl,
            &player,
            &format!(
                "Tried to sync time with an invalid time difference from UTC in minutes: {time_difference_from_utc_in_minutes}"
            ),
        )));
    }
    player.set_time_difference_from_utc_in_minutes(time_difference_from_utc_in_minutes);
    dsl.update_player_by_id(player)?;

    Ok(())
}
