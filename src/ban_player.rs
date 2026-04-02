use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::tables::player::{GetPlayerRowOptionById, Player, PlayerId, UpdatePlayerRowById};

pub fn ban_player(
    dsl: &DSL<'_, ReducerContext>,
    player_id: impl Into<PlayerId>,
    reason: &str,
) -> Result<Player, SpacetimeDSLError> {
    let mut player = dsl.get_player_by_id(player_id.into())?;

    player.set_is_banned(Some(reason.to_string()));

    let player = dsl.update_player_by_id(player)?;

    Ok(player)
}

pub fn return_error_if_player_is_banned(player: &Player) -> Result<(), SpacetimeDSLError> {
    match player.get_is_banned() {
        Some(reason) => Err(SpacetimeDSLError::Error(format!(
            "Player is banned. Reason: {reason}."
        ))),
        None => Ok(()),
    }
}
