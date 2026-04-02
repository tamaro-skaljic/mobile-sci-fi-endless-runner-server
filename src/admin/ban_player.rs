use spacetimedb::{Identity, ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    checks::is_admin_client::verify_admin_access, or_ok_on_cheat, tables::player::PlayerId,
};

#[reducer]
pub fn ban_player(
    ctx: &ReducerContext,
    player_id: Identity,
    reason: &str,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let player_id = PlayerId::new(player_id);

    or_ok_on_cheat!(verify_admin_access(&dsl, "ban players"));

    crate::ban_player::ban_player(&dsl, player_id, reason)?;

    Ok(())
}
