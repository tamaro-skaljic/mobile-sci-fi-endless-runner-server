use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    authenticated_player::get_authenticated_player,
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    checks::player_has_playthrough::player_has_playthrough,
    energy::{recalculate_energy_for_active_play, start_regen_after_playthrough},
    or_ok_on_cheat,
    scheduled_functions::energy_depletion::DeleteEnergyDepletionScheduleRowByPlayerId,
    tables::{
        energy::{GetEnergyRowOptionByPlayerId, UpdateEnergyRowByPlayerId},
        player::UpdatePlayerRowById,
        playthrough::{
            EndReason, GetPlaythroughRowOptionById, PauseReason, UpdatePlaythroughRowById,
        },
    },
};

#[reducer]
pub fn end_playthrough(
    ctx: &ReducerContext,
    end_reason: EndReason,
    score: u64,
    coins: u64,
    gems: u64,
    is_high_score: bool,
    levels: u32,
) -> Result<(), SpacetimeDSLError> {
    let (dsl, mut player) = get_authenticated_player(ctx)?;

    let last_playthrough_id = or_ok_on_cheat!(player_has_playthrough(&dsl, &player));
    let mut playthrough = dsl.get_playthrough_by_id(last_playthrough_id)?;

    if playthrough.get_end().is_some() {
        or_ok_on_cheat!(Err(cheat_attempt(
            &dsl,
            &player,
            "Tried to end a playthrough that already ended",
        )));
    }

    let now = dsl.ctx().timestamp;

    let is_paused = playthrough
        .get_pauses()
        .last()
        .is_some_and(|p| p.end.is_none());

    let current_pause_reason = if is_paused {
        Some(playthrough.get_pauses().last().unwrap().reason.clone())
    } else {
        None
    };

    or_ok_on_cheat!(validate_end_reason(
        &dsl,
        &player,
        &end_reason,
        &current_pause_reason
    ));

    let mut energy = dsl.get_energy_by_player_id(&player)?;

    if is_paused {
        let pauses = playthrough.get_pauses_mut();
        if let Some(last_pause) = pauses.last_mut() {
            last_pause.end = Some(now);
        }
    } else {
        recalculate_energy_for_active_play(&mut energy, now);
        dsl.delete_energy_depletion_schedule_by_player_id(&player)?;
    }

    start_regen_after_playthrough(&mut energy, &playthrough, now);
    dsl.update_energy_by_player_id(energy)?;

    playthrough.set_end(Some(now));
    playthrough.set_end_reason(Some(end_reason));
    playthrough.set_score(score);
    playthrough.set_coins(coins);
    playthrough.set_gems(gems);
    playthrough.set_is_high_score(is_high_score);
    playthrough.set_levels(levels);
    dsl.update_playthrough_by_id(playthrough)?;

    player.set_level(*player.get_level() + levels);
    dsl.update_player_by_id(player)?;

    Ok(())
}

fn validate_end_reason(
    dsl: &DSL<'_, ReducerContext>,
    player: &crate::tables::player::Player,
    end_reason: &EndReason,
    current_pause_reason: &Option<PauseReason>,
) -> Result<(), CheatOrError> {
    let is_paused = current_pause_reason.is_some();
    match end_reason {
        EndReason::UserRequest => {
            if !matches!(current_pause_reason, Some(PauseReason::Pause)) {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "UserRequest end reason requires an active Pause",
                ));
            }
        }
        EndReason::GameClose => {}
        EndReason::Death => {
            if is_paused {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "Death end reason requires active unpaused playthrough",
                ));
            }
        }
        EndReason::NoRevive => {
            if !matches!(current_pause_reason, Some(PauseReason::Revive)) {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "NoRevive end reason requires an active Revive pause",
                ));
            }
        }
        EndReason::OutOfEnergy => {
            if !matches!(current_pause_reason, Some(PauseReason::OutOfEnergy)) {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "OutOfEnergy end reason requires an active OutOfEnergy pause",
                ));
            }
        }
    }
    Ok(())
}
