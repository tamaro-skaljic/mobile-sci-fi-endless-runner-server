use spacetimedb::Timestamp;

use crate::constants::{ONE_DAY_IN_MICROS, ONE_MINUTE_IN_MICROS};

pub fn is_same_local_day(
    timestamp_a: Timestamp,
    timestamp_b: Timestamp,
    time_difference_from_utc_in_minutes: i16,
) -> bool {
    let offset_micros = time_difference_from_utc_in_minutes as i64 * ONE_MINUTE_IN_MICROS;

    let day_a = (timestamp_a.to_micros_since_unix_epoch() + offset_micros) / ONE_DAY_IN_MICROS;
    let day_b = (timestamp_b.to_micros_since_unix_epoch() + offset_micros) / ONE_DAY_IN_MICROS;

    day_a == day_b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp_from_micros(micros: i64) -> Timestamp {
        Timestamp::from_micros_since_unix_epoch(micros)
    }

    #[test]
    fn same_timestamp_is_same_day() {
        let timestamp = timestamp_from_micros(1_000_000_000_000);
        assert!(is_same_local_day(timestamp, timestamp, 0));
    }

    #[test]
    fn same_utc_day_different_times() {
        let morning = timestamp_from_micros(ONE_DAY_IN_MICROS * 100);
        let evening = timestamp_from_micros(ONE_DAY_IN_MICROS * 100 + ONE_DAY_IN_MICROS - 1);
        assert!(is_same_local_day(morning, evening, 0));
    }

    #[test]
    fn different_utc_days() {
        let day_one = timestamp_from_micros(ONE_DAY_IN_MICROS * 100);
        let day_two = timestamp_from_micros(ONE_DAY_IN_MICROS * 101);
        assert!(!is_same_local_day(day_one, day_two, 0));
    }

    #[test]
    fn timezone_offset_shifts_day_boundary() {
        let utc_day_boundary = ONE_DAY_IN_MICROS * 100;
        let just_before_utc_midnight =
            timestamp_from_micros(utc_day_boundary - ONE_MINUTE_IN_MICROS);
        let just_after_utc_midnight =
            timestamp_from_micros(utc_day_boundary + ONE_MINUTE_IN_MICROS);

        assert!(!is_same_local_day(
            just_before_utc_midnight,
            just_after_utc_midnight,
            0
        ));

        assert!(is_same_local_day(
            just_before_utc_midnight,
            just_after_utc_midnight,
            120
        ));
    }

    #[test]
    fn negative_timezone_offset() {
        let utc_day_boundary = ONE_DAY_IN_MICROS * 100;
        let early_utc = timestamp_from_micros(utc_day_boundary + ONE_MINUTE_IN_MICROS);
        let later_utc = timestamp_from_micros(utc_day_boundary + 2 * 60 * ONE_MINUTE_IN_MICROS);

        assert!(is_same_local_day(early_utc, later_utc, 0));

        assert!(!is_same_local_day(early_utc, later_utc, -60));
    }
}
