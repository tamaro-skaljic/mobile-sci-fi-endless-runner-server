use spacetimedb::{Identity, ReducerContext, ScheduleAt, reducer, table};
use spacetimedsl::prelude::*;

use crate::tables::energy::{GetEnergyRowOptionByPlayerId, UpdateEnergyRowByPlayerId};

#[dsl(plural_name = energy_depletion_schedules, method(update = false, delete = true))]
#[table(accessor = energy_depletion_schedule, scheduled(on_energy_depleted), private)]
pub struct EnergyDepletionSchedule {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,

    scheduled_at: ScheduleAt,

    #[unique]
    #[use_wrapper(crate::tables::player::PlayerId)]
    #[foreign_key(path = crate::tables::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,
}

#[reducer]
pub fn on_energy_depleted(
    ctx: &ReducerContext,
    job: EnergyDepletionSchedule,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let mut energy = dsl.get_energy_by_player_id(job.get_player_id())?;

    energy.set_energy(0);
    energy.set_energy_boundary_reached_at(None);
    energy.set_last_energy_calculation_at(dsl.ctx().timestamp);

    dsl.update_energy_by_player_id(energy)?;

    Ok(())
}
