use spacetimedb::{Identity, ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    checks::is_admin_client::verify_admin_access,
    or_ok_on_cheat,
    tables::player::{GetPlayerRowOptionById, PlayerId, UpdatePlayerRowById},
};

#[reducer]
pub fn unban_player(ctx: &ReducerContext, player_id: Identity) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    or_ok_on_cheat!(verify_admin_access(&dsl, "unban players"));

    let mut player = dsl.get_player_by_id(PlayerId::new(player_id))?;

    player.set_is_banned(None);

    dsl.update_player_by_id(player)?;

    Ok(())
}
