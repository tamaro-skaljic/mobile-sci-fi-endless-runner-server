use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    authenticated_player::get_authenticated_player,
    checks::{
        player_has_playthrough::player_has_playthrough,
        playthrough_is_active_and_unpaused::playthrough_is_active_and_unpaused,
    },
    energy::recalculate_energy_for_active_play,
    or_ok_on_cheat,
    scheduled_functions::energy_depletion::DeleteEnergyDepletionScheduleRowByPlayerId,
    tables::{
        energy::{GetEnergyRowOptionByPlayerId, UpdateEnergyRowByPlayerId},
        playthrough::{
            GetPlaythroughRowOptionById, PauseEntry, PauseReason, UpdatePlaythroughRowById,
        },
    },
};

#[reducer]
pub fn pause_playthrough(
    ctx: &ReducerContext,
    pause_reason: PauseReason,
) -> Result<(), SpacetimeDSLError> {
    let (dsl, player) = get_authenticated_player(ctx)?;

    let last_playthrough_id = or_ok_on_cheat!(player_has_playthrough(&dsl, &player));
    let mut playthrough = dsl.get_playthrough_by_id(last_playthrough_id)?;

    or_ok_on_cheat!(playthrough_is_active_and_unpaused(
        &dsl,
        &player,
        &playthrough
    ));

    let now = dsl.ctx().timestamp;
    let mut energy = dsl.get_energy_by_player_id(&player)?;

    recalculate_energy_for_active_play(&mut energy, now);

    energy.set_energy_boundary_reached_at(None);
    energy.set_last_energy_calculation_at(now);

    dsl.update_energy_by_player_id(energy)?;

    dsl.delete_energy_depletion_schedule_by_player_id(&player)?;

    playthrough.get_pauses_mut().push(PauseEntry {
        begin: now,
        end: None,
        reason: pause_reason,
    });
    dsl.update_playthrough_by_id(playthrough)?;

    Ok(())
}
