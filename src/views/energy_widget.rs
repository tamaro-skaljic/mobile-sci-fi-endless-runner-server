use spacetimedb::{SpacetimeType, TimeDuration, ViewContext};
use spacetimedsl::prelude::*;

use crate::{
    constants::{
        ENERGY_CONSUMPTION_INTERVAL_MICROS, ENERGY_REGENERATION_INTERVAL_MICROS, MAX_ENERGY,
    },
    energy::calculation::{calculate_energy_during_active_play, recalculate_energy_for_idle},
    tables::{
        energy::{EnergyChange, energy__view},
        player::player__view,
        playthrough::playthrough__view,
    },
};

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct EnergyWidget {
    pub current_energy: u8,
    pub max_energy: u8,
    pub next_energy_change_at: Option<EnergyChange>,
    pub regeneration_interval_micros: i64,
    pub consumption_interval_micros: i64,
}

#[spacetimedb::view(accessor = energy_widget, public)]
pub fn energy_widget(ctx: &ViewContext) -> Option<EnergyWidget> {
    let player = ctx.db.player().id().find(ctx.sender())?;
    let energy = ctx.db.energy().player_id().find(ctx.sender())?;
    let now = ctx.timestamp().ok()?;

    let (current_energy, next_energy_change_at) = match player.get_last_playthrough_id() {
        Some(playthrough_id) => {
            let playthrough = ctx.db.playthrough().id().find(playthrough_id.value())?;

            if playthrough.get_end().is_some() {
                // idle: playthrough ended
                calculate_idle_state(energy, now)
            } else {
                // active playthrough
                let is_paused = playthrough
                    .get_pauses()
                    .last()
                    .is_some_and(|pause| pause.end.is_none());

                if is_paused {
                    // paused: energy is already current, no timer
                    (*energy.get_energy(), None)
                } else {
                    // active unpaused: compute with ceiling division
                    calculate_energy_during_active_play(
                        *energy.get_energy(),
                        *energy.get_last_energy_calculation_at(),
                        now,
                    )
                }
            }
        }
        // idle: no playthrough ever started
        None => calculate_idle_state(energy, now),
    };

    Some(EnergyWidget {
        current_energy,
        max_energy: MAX_ENERGY,
        next_energy_change_at,
        regeneration_interval_micros: ENERGY_REGENERATION_INTERVAL_MICROS,
        consumption_interval_micros: ENERGY_CONSUMPTION_INTERVAL_MICROS,
    })
}

fn calculate_idle_state(
    mut energy: crate::tables::energy::Energy,
    now: spacetimedb::Timestamp,
) -> (u8, Option<EnergyChange>) {
    recalculate_energy_for_idle(&mut energy, now);
    let current_energy = *energy.get_energy();

    if current_energy >= MAX_ENERGY {
        return (current_energy, None);
    }

    // next regeneration tick
    let next_regeneration_at = *energy.get_last_energy_regeneration_at()
        + TimeDuration::from_micros(ENERGY_REGENERATION_INTERVAL_MICROS);

    (
        current_energy,
        Some(EnergyChange::Increasing(next_regeneration_at)),
    )
}
