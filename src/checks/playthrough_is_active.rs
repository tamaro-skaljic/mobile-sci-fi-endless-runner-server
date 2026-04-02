use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    tables::{player::Player, playthrough::Playthrough},
};

pub fn playthrough_is_active(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    playthrough: &Playthrough,
) -> Result<(), CheatOrError> {
    if playthrough.get_end().is_some() {
        return Err(cheat_attempt(dsl, player, "Playthrough already ended"));
    }
    Ok(())
}
