use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::tables::player::{GetPlayerRowOptionById, PlayerId, UpdatePlayerRowById};

#[reducer(client_disconnected)]
pub fn disconnect_client(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let player = dsl.get_player_by_id(PlayerId::new(dsl.ctx().sender()))?;

    // Trigger modified_at update to record last disconnect time
    dsl.update_player_by_id(player)?;

    Ok(())
}
