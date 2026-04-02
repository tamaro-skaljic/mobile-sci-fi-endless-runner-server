use spacetimedb::{TimeDuration, Timestamp};

use crate::constants::{
    ENERGY_CONSUMPTION_INTERVAL_MICROS, ENERGY_REGENERATION_INTERVAL_MICROS, MAX_ENERGY,
};
use crate::tables::energy::{Energy, EnergyChange};

pub fn timestamp_diff_micros(later: Timestamp, earlier: Timestamp) -> i64 {
    later.to_micros_since_unix_epoch() - earlier.to_micros_since_unix_epoch()
}

pub fn recalculate_energy_for_idle(energy: &mut Energy, now: Timestamp) {
    let elapsed_micros = timestamp_diff_micros(now, *energy.get_last_energy_regeneration_at());
    let regen_ticks = elapsed_micros / ENERGY_REGENERATION_INTERVAL_MICROS;

    if regen_ticks > 0 {
        let new_energy = (*energy.get_energy() as i64 + regen_ticks).min(MAX_ENERGY as i64) as u8;
        energy.set_energy(new_energy);
        energy.set_last_energy_regeneration_at(
            *energy.get_last_energy_regeneration_at()
                + TimeDuration::from_micros(regen_ticks * ENERGY_REGENERATION_INTERVAL_MICROS),
        );
    }

    energy.set_last_energy_calculation_at(now);
}

pub fn recalculate_energy_for_active_play(energy: &mut Energy, now: Timestamp) {
    let (new_energy, _) = calculate_energy_during_active_play(
        *energy.get_energy(),
        *energy.get_last_energy_calculation_at(),
        now,
    );

    energy.set_energy(new_energy);
    energy.set_last_energy_calculation_at(now);
}

pub fn calculate_depletion_timestamp(energy_amount: u8, now: Timestamp) -> Option<EnergyChange> {
    if energy_amount == 0 {
        return None;
    }
    let micros = (energy_amount as i64 - 1) * ENERGY_CONSUMPTION_INTERVAL_MICROS + 1;
    Some(EnergyChange::Decreasing(
        now + TimeDuration::from_micros(micros),
    ))
}

pub fn calculate_energy_during_active_play(
    stored_energy: u8,
    last_energy_calculation_at: Timestamp,
    now: Timestamp,
) -> (u8, Option<EnergyChange>) {
    let elapsed_micros = timestamp_diff_micros(now, last_energy_calculation_at);

    // ceiling division: any started interval counts as one consumed unit
    let consumed = if elapsed_micros > 0 {
        (elapsed_micros + ENERGY_CONSUMPTION_INTERVAL_MICROS - 1)
            / ENERGY_CONSUMPTION_INTERVAL_MICROS
    } else {
        0
    };

    let current_energy = (stored_energy as i64 - consumed).max(0) as u8;

    if current_energy == 0 {
        return (0, None);
    }

    // next consumption boundary: last_calc + consumed * interval
    let next_change_at = last_energy_calculation_at
        + TimeDuration::from_micros(consumed * ENERGY_CONSUMPTION_INTERVAL_MICROS);

    (
        current_energy,
        Some(EnergyChange::Decreasing(next_change_at)),
    )
}

pub fn calculate_regen_completion_timestamp(
    current_energy: u8,
    now: Timestamp,
) -> Option<EnergyChange> {
    if current_energy < MAX_ENERGY {
        let remaining = (MAX_ENERGY - current_energy) as i64 * ENERGY_REGENERATION_INTERVAL_MICROS;
        Some(EnergyChange::Increasing(
            now + TimeDuration::from_micros(remaining),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recalculate_idle_no_regen_when_less_than_interval() {
        let elapsed_micros = 100_000_000; // 100s < 180s
        let regen_ticks = elapsed_micros / ENERGY_REGENERATION_INTERVAL_MICROS;
        assert_eq!(regen_ticks, 0);
    }

    #[test]
    fn recalculate_idle_one_regen_tick() {
        let elapsed_micros = 180_000_000; // exactly 180s
        let regen_ticks = elapsed_micros / ENERGY_REGENERATION_INTERVAL_MICROS;
        assert_eq!(regen_ticks, 1);
    }

    #[test]
    fn recalculate_idle_partial_tick_floors() {
        let elapsed_micros = 359_999_999; // just under 360s
        let regen_ticks = elapsed_micros / ENERGY_REGENERATION_INTERVAL_MICROS;
        assert_eq!(regen_ticks, 1);
    }

    #[test]
    fn recalculate_idle_caps_at_max() {
        let current_energy = 58u8;
        let regen_ticks = 5i64;
        let new_energy = (current_energy as i64 + regen_ticks).min(MAX_ENERGY as i64) as u8;
        assert_eq!(new_energy, MAX_ENERGY);
    }

    #[test]
    fn consumption_ceil_zero_point_one_seconds_costs_one() {
        let elapsed_micros = 10_000; // 0.01s
        let consumed = (elapsed_micros + ENERGY_CONSUMPTION_INTERVAL_MICROS - 1)
            / ENERGY_CONSUMPTION_INTERVAL_MICROS;
        assert_eq!(consumed, 1);
    }

    #[test]
    fn consumption_ceil_exactly_one_interval_costs_one() {
        let elapsed_micros = ENERGY_CONSUMPTION_INTERVAL_MICROS;
        let consumed = (elapsed_micros + ENERGY_CONSUMPTION_INTERVAL_MICROS - 1)
            / ENERGY_CONSUMPTION_INTERVAL_MICROS;
        assert_eq!(consumed, 1);
    }

    #[test]
    fn consumption_ceil_just_over_one_interval_costs_two() {
        let elapsed_micros = ENERGY_CONSUMPTION_INTERVAL_MICROS + 10_000;
        let consumed = (elapsed_micros + ENERGY_CONSUMPTION_INTERVAL_MICROS - 1)
            / ENERGY_CONSUMPTION_INTERVAL_MICROS;
        assert_eq!(consumed, 2);
    }

    #[test]
    fn consumption_saturates_at_zero() {
        let current_energy = 2i64;
        let consumed = 5i64;
        let new_energy = (current_energy - consumed).max(0) as u8;
        assert_eq!(new_energy, 0);
    }

    #[test]
    fn fractional_regen_progress_preserved() {
        let last_regen_at = 1_000_000_000i64;
        let playthrough_begin = last_regen_at + 170_000_000;
        let now = playthrough_begin + 500_000_000;

        let fractional_progress = playthrough_begin - last_regen_at;
        let new_last_regen = now - fractional_progress;

        let time_until_next_regen = (new_last_regen + ENERGY_REGENERATION_INTERVAL_MICROS) - now;
        assert_eq!(time_until_next_regen, 10_000_000);
    }

    fn timestamp(micros: i64) -> Timestamp {
        Timestamp::from_micros_since_unix_epoch(micros)
    }

    #[test]
    fn depletion_zero_energy_returns_none() {
        let result = calculate_depletion_timestamp(0, timestamp(1_000_000));
        assert_eq!(result, None);
    }

    #[test]
    fn depletion_one_energy_depletes_in_one_microsecond() {
        let now = timestamp(1_000_000);
        let result = calculate_depletion_timestamp(1, now);

        // ceiling: ceil(1μs / interval) = 1 consumed → energy = 0
        let expected = timestamp(1_000_000 + 1);
        assert_eq!(result, Some(EnergyChange::Decreasing(expected)));
    }

    #[test]
    fn depletion_two_energy_depletes_at_one_interval_plus_one() {
        let now = timestamp(1_000_000);
        let result = calculate_depletion_timestamp(2, now);

        // second unit consumed when ceil(elapsed/interval) = 2
        // that happens at elapsed = interval + 1
        let expected = timestamp(1_000_000 + ENERGY_CONSUMPTION_INTERVAL_MICROS + 1);
        assert_eq!(result, Some(EnergyChange::Decreasing(expected)));
    }

    #[test]
    fn depletion_consistent_with_ceiling_consumption() {
        let now = timestamp(1_000_000);
        let energy_amount: u8 = 10;
        let result = calculate_depletion_timestamp(energy_amount, now);

        // energy reaches 0 when ceil(elapsed/interval) = energy_amount
        // that happens at elapsed = (energy_amount - 1) * interval + 1
        let expected_micros = (energy_amount as i64 - 1) * ENERGY_CONSUMPTION_INTERVAL_MICROS + 1;
        let expected = timestamp(1_000_000 + expected_micros);
        assert_eq!(result, Some(EnergyChange::Decreasing(expected)));

        // verify: at the depletion time, ceiling division says consumed = energy_amount
        let elapsed_at_depletion = expected_micros;
        let consumed = (elapsed_at_depletion + ENERGY_CONSUMPTION_INTERVAL_MICROS - 1)
            / ENERGY_CONSUMPTION_INTERVAL_MICROS;
        assert_eq!(consumed, energy_amount as i64);
    }

    #[test]
    fn depletion_one_microsecond_before_depletion_energy_is_still_one() {
        let energy_amount: u8 = 5;

        // at (energy_amount - 1) * interval, consumed should be energy_amount - 1
        let elapsed = (energy_amount as i64 - 1) * ENERGY_CONSUMPTION_INTERVAL_MICROS;
        let consumed =
            (elapsed + ENERGY_CONSUMPTION_INTERVAL_MICROS - 1) / ENERGY_CONSUMPTION_INTERVAL_MICROS;
        assert_eq!(consumed, energy_amount as i64 - 1);
    }

    #[test]
    fn active_play_zero_elapsed_returns_stored_energy() {
        let now = timestamp(1_000_000);
        let (energy, next_change) = calculate_energy_during_active_play(10, now, now);

        assert_eq!(energy, 10);
        assert_eq!(
            next_change,
            Some(EnergyChange::Decreasing(timestamp(1_000_000)))
        );
    }

    #[test]
    fn active_play_mid_interval_consumes_one() {
        let last_calc = timestamp(1_000_000);
        let now = timestamp(1_000_000 + ENERGY_CONSUMPTION_INTERVAL_MICROS / 2);
        let (energy, next_change) = calculate_energy_during_active_play(10, last_calc, now);

        assert_eq!(energy, 9);
        let expected_next = timestamp(1_000_000 + ENERGY_CONSUMPTION_INTERVAL_MICROS);
        assert_eq!(next_change, Some(EnergyChange::Decreasing(expected_next)));
    }

    #[test]
    fn active_play_exactly_at_interval_boundary_consumes_one() {
        let last_calc = timestamp(1_000_000);
        let now = timestamp(1_000_000 + ENERGY_CONSUMPTION_INTERVAL_MICROS);
        let (energy, next_change) = calculate_energy_during_active_play(10, last_calc, now);

        // ceil(interval / interval) = 1
        assert_eq!(energy, 9);
        let expected_next = timestamp(1_000_000 + ENERGY_CONSUMPTION_INTERVAL_MICROS);
        assert_eq!(next_change, Some(EnergyChange::Decreasing(expected_next)));
    }

    #[test]
    fn active_play_just_past_interval_consumes_two() {
        let last_calc = timestamp(1_000_000);
        let now = timestamp(1_000_000 + ENERGY_CONSUMPTION_INTERVAL_MICROS + 1);
        let (energy, next_change) = calculate_energy_during_active_play(10, last_calc, now);

        // ceil((interval + 1) / interval) = 2
        assert_eq!(energy, 8);
        let expected_next = timestamp(1_000_000 + 2 * ENERGY_CONSUMPTION_INTERVAL_MICROS);
        assert_eq!(next_change, Some(EnergyChange::Decreasing(expected_next)));
    }

    #[test]
    fn active_play_energy_saturates_at_zero() {
        let last_calc = timestamp(1_000_000);
        let elapsed = 100 * ENERGY_CONSUMPTION_INTERVAL_MICROS;
        let now = timestamp(1_000_000 + elapsed);
        let (energy, next_change) = calculate_energy_during_active_play(5, last_calc, now);

        assert_eq!(energy, 0);
        assert_eq!(next_change, None);
    }

    #[test]
    fn active_play_zero_stored_energy_returns_none() {
        let now = timestamp(1_000_000);
        let (energy, next_change) = calculate_energy_during_active_play(0, now, now);

        assert_eq!(energy, 0);
        assert_eq!(next_change, None);
    }
}
