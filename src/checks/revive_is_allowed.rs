use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    checks::{
        player_has_playthrough::player_has_playthrough,
        playthrough_is_in_pause::playthrough_is_in_pause_reason,
    },
    constants::MAX_REVIVES_PER_PLAYTHROUGH,
    tables::{
        player::{GetPlayerRowOptionById, PlayerId},
        playthrough::{GetPlaythroughRowOptionById, PauseReason, Playthrough},
    },
};

pub fn revive_is_allowed(
    dsl: &DSL<'_, ReducerContext>,
    player_id: PlayerId,
) -> Result<(Playthrough, u8), CheatOrError> {
    let player = dsl.get_player_by_id(&player_id)?;

    let last_playthrough_id = player_has_playthrough(dsl, &player)?;
    let active_playthrough = dsl.get_playthrough_by_id(last_playthrough_id)?;

    playthrough_is_in_pause_reason(dsl, &player, &active_playthrough, &PauseReason::Revive)?;

    let revives_in_this_playthrough = *active_playthrough.get_revive_count();

    if revives_in_this_playthrough >= MAX_REVIVES_PER_PLAYTHROUGH {
        return Err(cheat_attempt(dsl, &player, "Max revives reached"));
    };

    Ok((active_playthrough, revives_in_this_playthrough))
}
