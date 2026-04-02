use spacetimedb::{ReducerContext, ScheduleAt, TimeDuration, Timestamp};
use spacetimedsl::prelude::*;

use crate::scheduled_functions::energy_depletion::{
    CreateEnergyDepletionSchedule, CreateEnergyDepletionScheduleRow,
};
use crate::tables::energy::Energy;
use crate::tables::player::PlayerId;
use crate::tables::playthrough::{PauseReason, Playthrough};

use super::calculation::{
    calculate_depletion_timestamp, calculate_regen_completion_timestamp, timestamp_diff_micros,
};

pub fn schedule_energy_depletion(
    dsl: &DSL<'_, ReducerContext>,
    player_id: impl Into<PlayerId>,
    energy: &Energy,
    now: Timestamp,
) -> Result<(), SpacetimeDSLError> {
    if let Some(energy_change) = calculate_depletion_timestamp(*energy.get_energy(), now) {
        dsl.create_energy_depletion_schedule(CreateEnergyDepletionSchedule {
            scheduled_at: ScheduleAt::Time(energy_change.timestamp()),
            player_id: player_id.into(),
        })?;
    }

    Ok(())
}

pub fn start_regen_after_playthrough(
    energy: &mut Energy,
    playthrough: &Playthrough,
    now: Timestamp,
) {
    let fractional_progress = timestamp_diff_micros(
        *playthrough.get_begin(),
        *energy.get_last_energy_regeneration_at(),
    );
    let new_last_energy_regeneration_at = now - TimeDuration::from_micros(fractional_progress);
    energy.set_last_energy_regeneration_at(new_last_energy_regeneration_at);

    energy.set_energy_boundary_reached_at(calculate_regen_completion_timestamp(
        *energy.get_energy(),
        now,
    ));

    energy.set_last_energy_calculation_at(now);
}

pub fn is_in_out_of_energy_pause(playthrough: &Playthrough) -> bool {
    let pauses = playthrough.get_pauses();
    if let Some(last_pause) = pauses.last() {
        last_pause.end.is_none() && last_pause.reason == PauseReason::OutOfEnergy
    } else {
        false
    }
}
