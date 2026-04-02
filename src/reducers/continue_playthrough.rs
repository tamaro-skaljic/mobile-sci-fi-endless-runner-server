use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    authenticated_player::get_authenticated_player,
    cheat_attempt_log::cheat_attempt,
    checks::{
        player_has_playthrough::player_has_playthrough,
        playthrough_is_paused::playthrough_is_paused,
    },
    energy::{calculate_depletion_timestamp, schedule_energy_depletion},
    or_ok_on_cheat,
    tables::{
        energy::{GetEnergyRowOptionByPlayerId, UpdateEnergyRowByPlayerId},
        playthrough::{GetPlaythroughRowOptionById, PauseReason, UpdatePlaythroughRowById},
    },
};

#[reducer]
pub fn continue_playthrough(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let (dsl, player) = get_authenticated_player(ctx)?;

    let last_playthrough_id = or_ok_on_cheat!(player_has_playthrough(&dsl, &player));
    let mut playthrough = dsl.get_playthrough_by_id(last_playthrough_id)?;

    or_ok_on_cheat!(playthrough_is_paused(&dsl, &player, &playthrough));

    let now = dsl.ctx().timestamp;
    let mut energy = dsl.get_energy_by_player_id(&player)?;

    let last_pause = playthrough.get_pauses().last().unwrap();
    if last_pause.reason == PauseReason::OutOfEnergy && *energy.get_energy() == 0 {
        or_ok_on_cheat!(Err(cheat_attempt(
            &dsl,
            &player,
            "Tried to continue from OutOfEnergy pause with 0 energy",
        )));
    }

    energy.set_last_energy_calculation_at(now);
    energy.set_energy_boundary_reached_at(calculate_depletion_timestamp(*energy.get_energy(), now));

    dsl.update_energy_by_player_id(energy.clone())?;

    schedule_energy_depletion(&dsl, &player, &energy, now)?;

    let pauses = playthrough.get_pauses_mut();
    if let Some(last_pause) = pauses.last_mut() {
        last_pause.end = Some(now);
    }
    dsl.update_playthrough_by_id(playthrough)?;

    Ok(())
}
