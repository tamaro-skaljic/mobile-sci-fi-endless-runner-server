use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    tables::{player::Player, playthrough::Playthrough},
};

use super::playthrough_is_active::playthrough_is_active;

pub fn playthrough_is_paused(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    playthrough: &Playthrough,
) -> Result<(), CheatOrError> {
    playthrough_is_active(dsl, player, playthrough)?;

    match playthrough.get_pauses().last() {
        Some(last_pause) if last_pause.end.is_none() => Ok(()),
        _ => Err(cheat_attempt(dsl, player, "Playthrough is not paused")),
    }
}
