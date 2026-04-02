use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    authenticated_player::get_authenticated_player,
    cheat_attempt_log::cheat_attempt,
    energy::{
        calculate_depletion_timestamp, recalculate_energy_for_idle, schedule_energy_depletion,
    },
    or_ok_on_cheat,
    tables::{
        energy::{GetEnergyRowOptionByPlayerId, UpdateEnergyRowByPlayerId},
        player::UpdatePlayerRowById,
        playthrough::{CreatePlaythrough, CreatePlaythroughRow, GetPlaythroughRowOptionById},
    },
};

#[reducer]
pub fn begin_playthrough(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let (dsl, mut player) = get_authenticated_player(ctx)?;

    if let Some(last_playthrough_id) = player.get_last_playthrough_id() {
        let last_playthrough = dsl.get_playthrough_by_id(last_playthrough_id)?;
        if last_playthrough.get_end().is_none() {
            or_ok_on_cheat!(Err(cheat_attempt(
                &dsl,
                &player,
                "Tried to begin a playthrough while one is already active or paused",
            )));
        }
    }

    let now = dsl.ctx().timestamp;
    let mut energy = dsl.get_energy_by_player_id(&player)?;

    recalculate_energy_for_idle(&mut energy, now);

    if *energy.get_energy() == 0 {
        or_ok_on_cheat!(Err(cheat_attempt(
            &dsl,
            &player,
            "Tried to begin a playthrough with 0 energy",
        )));
    }

    energy.set_energy(energy.get_energy() - 1);

    energy.set_energy_boundary_reached_at(calculate_depletion_timestamp(*energy.get_energy(), now));

    dsl.update_energy_by_player_id(energy.clone())?;

    let playthrough = dsl.create_playthrough(CreatePlaythrough {
        player_id: player.get_id(),
        score: 0,
        coins: 0,
        gems: 0,
        is_high_score: false,
        revive_count: 0,
        coins_doubled: false,
        energy_purchases: 0,
        levels: 0,
        pauses: vec![],
        end_reason: None,
        begin: now,
        end: None,
    })?;

    player.set_last_playthrough_id(playthrough.get_id());
    schedule_energy_depletion(&dsl, &player, &energy, now)?;
    dsl.update_player_by_id(player)?;

    Ok(())
}
