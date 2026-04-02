use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::ban_player::return_error_if_player_is_banned;
use crate::tables::player::{GetPlayerRowOptionById, Player, PlayerId};

pub fn get_authenticated_player(
    ctx: &ReducerContext,
) -> Result<(DSL<'_, ReducerContext>, Player), SpacetimeDSLError> {
    let dsl = dsl(ctx);
    let sender = dsl.ctx().sender();
    let player_id = PlayerId::new(sender);
    let player = dsl.get_player_by_id(&player_id)?;
    return_error_if_player_is_banned(&player)?;
    Ok((dsl, player))
}
