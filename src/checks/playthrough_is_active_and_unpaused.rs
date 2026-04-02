use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    tables::{player::Player, playthrough::Playthrough},
};

use super::playthrough_is_active::playthrough_is_active;

pub fn playthrough_is_active_and_unpaused(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    playthrough: &Playthrough,
) -> Result<(), CheatOrError> {
    playthrough_is_active(dsl, player, playthrough)?;

    if let Some(last_pause) = playthrough.get_pauses().last()
        && last_pause.end.is_none()
    {
        return Err(cheat_attempt(dsl, player, "Playthrough is paused"));
    }

    Ok(())
}
