use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    tables::{player::Player, playthrough::PlaythroughId},
};

pub fn player_has_playthrough(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
) -> Result<PlaythroughId, CheatOrError> {
    match player.get_last_playthrough_id() {
        Some(playthrough_id) => Ok(playthrough_id),
        None => Err(cheat_attempt(dsl, player, "Never had a playthrough")),
    }
}
