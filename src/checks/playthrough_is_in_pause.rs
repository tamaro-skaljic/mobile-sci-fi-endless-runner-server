use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    tables::{
        player::Player,
        playthrough::{PauseReason, Playthrough},
    },
};

use super::playthrough_is_active::playthrough_is_active;

pub fn playthrough_is_in_pause_reason(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    playthrough: &Playthrough,
    expected: &PauseReason,
) -> Result<(), CheatOrError> {
    playthrough_is_active(dsl, player, playthrough)?;

    match playthrough.get_pauses().last() {
        Some(last_pause) if last_pause.end.is_none() && last_pause.reason == *expected => Ok(()),
        _ => Err(cheat_attempt(
            dsl,
            player,
            &format!("Playthrough is not in a {expected:?} pause"),
        )),
    }
}
