use spacetimedb::{SpacetimeType, Timestamp, ViewContext};
use spacetimedsl::prelude::*;

use crate::{
    checks::is_same_local_day::is_same_local_day,
    daily_reward_gems::daily_reward_gems,
    tables::player::{Player, player__view},
};

const STREAK_GOAL_DAY: u16 = 7;
const ELLIPSIS_DISPLAY_NEXT_DAY_CUTOFF: u16 = 6;
const NEXT_DAY_DISPLAY_CUTOFF: u16 = 9;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct DailyRewards {
    pub daily_reward_widgets_shown: Vec<DailyRewardWidget>,
    pub show_three_points_before_last_daily_reward_widget: bool,
    pub can_claim_first_daily_reward_widget: bool,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct DailyRewardWidget {
    pub day: u16,
    pub gems: u16,
}

#[spacetimedb::view(accessor = daily_reward_widgets, public)]
pub fn daily_reward_widgets(ctx: &ViewContext) -> Option<DailyRewards> {
    let player = ctx.db.player().id().find(ctx.sender())?;

    let current_timestamp = ctx.timestamp().ok()?;
    let can_claim_first_daily_reward_widget = is_daily_reward_claimable(&player, current_timestamp);

    Some(build_daily_rewards(
        *player.get_number_of_claimed_daily_rewards(),
        can_claim_first_daily_reward_widget,
    ))
}

pub fn build_daily_rewards(
    number_of_claimed_daily_rewards: u16,
    can_claim_first_daily_reward_widget: bool,
) -> DailyRewards {
    let current_day = number_of_claimed_daily_rewards.saturating_add(1);
    let next_day = current_day.saturating_add(1);

    let mut daily_reward_widgets_shown = vec![DailyRewardWidget::for_day(current_day)];

    if next_day < NEXT_DAY_DISPLAY_CUTOFF {
        daily_reward_widgets_shown.push(DailyRewardWidget::for_day(next_day));
    }

    let show_three_points_before_last_daily_reward_widget =
        next_day < ELLIPSIS_DISPLAY_NEXT_DAY_CUTOFF;

    if next_day < STREAK_GOAL_DAY {
        daily_reward_widgets_shown.push(DailyRewardWidget::for_day(STREAK_GOAL_DAY));
    }

    DailyRewards {
        daily_reward_widgets_shown,
        show_three_points_before_last_daily_reward_widget,
        can_claim_first_daily_reward_widget,
    }
}

pub fn is_daily_reward_claimable(player: &Player, current_timestamp: Timestamp) -> bool {
    let last_claimed_at = player.get_last_daily_reward_claimed_at();
    can_claim_daily_reward(
        last_claimed_at.as_ref(),
        current_timestamp,
        *player.get_time_difference_from_utc_in_minutes(),
    )
}

pub fn can_claim_daily_reward(
    last_daily_reward_claimed_at: Option<&Timestamp>,
    current_timestamp: Timestamp,
    time_difference_from_utc_in_minutes: i16,
) -> bool {
    let last_claimed_at = match last_daily_reward_claimed_at {
        Some(timestamp) => *timestamp,
        None => return true,
    };

    !is_same_local_day(
        last_claimed_at,
        current_timestamp,
        time_difference_from_utc_in_minutes,
    )
}

impl DailyRewardWidget {
    fn for_day(day: u16) -> Self {
        Self {
            day,
            // All daily reward values are at most 500 gems, which fits within u16.
            gems: daily_reward_gems(day.saturating_sub(1)) as u16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{ONE_DAY_IN_MICROS, ONE_MINUTE_IN_MICROS};

    fn timestamp(micros: i64) -> Timestamp {
        Timestamp::from_micros_since_unix_epoch(micros)
    }

    #[test]
    fn day_1_shows_current_next_and_seventh_day_with_ellipsis() {
        let result = build_daily_rewards(0, false);

        assert_eq!(
            result,
            DailyRewards {
                daily_reward_widgets_shown: vec![
                    DailyRewardWidget { day: 1, gems: 10 },
                    DailyRewardWidget { day: 2, gems: 30 },
                    DailyRewardWidget { day: 7, gems: 500 },
                ],
                show_three_points_before_last_daily_reward_widget: true,
                can_claim_first_daily_reward_widget: false,
            }
        );
    }

    #[test]
    fn day_5_shows_current_next_and_seventh_day_without_ellipsis() {
        let result = build_daily_rewards(4, false);

        assert_eq!(
            result,
            DailyRewards {
                daily_reward_widgets_shown: vec![
                    DailyRewardWidget { day: 5, gems: 150 },
                    DailyRewardWidget { day: 6, gems: 250 },
                    DailyRewardWidget { day: 7, gems: 500 },
                ],
                show_three_points_before_last_daily_reward_widget: false,
                can_claim_first_daily_reward_widget: false,
            }
        );
    }

    #[test]
    fn day_6_shows_current_and_seventh_day_without_ellipsis() {
        let result = build_daily_rewards(5, false);

        assert_eq!(
            result,
            DailyRewards {
                daily_reward_widgets_shown: vec![
                    DailyRewardWidget { day: 6, gems: 250 },
                    DailyRewardWidget { day: 7, gems: 500 },
                ],
                show_three_points_before_last_daily_reward_widget: false,
                can_claim_first_daily_reward_widget: false,
            }
        );
    }

    #[test]
    fn can_claim_when_never_claimed_before() {
        assert!(can_claim_daily_reward(None, timestamp(1_000), 0));
    }

    #[test]
    fn can_claim_when_last_claim_was_on_a_different_day() {
        let yesterday = timestamp(ONE_DAY_IN_MICROS * 100);
        let today = timestamp(ONE_DAY_IN_MICROS * 101);

        assert!(can_claim_daily_reward(Some(&yesterday), today, 0));
    }

    #[test]
    fn cannot_claim_when_already_claimed_today() {
        let morning = timestamp(ONE_DAY_IN_MICROS * 100);
        let evening = timestamp(ONE_DAY_IN_MICROS * 100 + ONE_DAY_IN_MICROS - 1);

        assert!(!can_claim_daily_reward(Some(&morning), evening, 0));
    }

    #[test]
    fn can_claim_respects_player_timezone() {
        let utc_day_boundary = ONE_DAY_IN_MICROS * 100;
        let before_utc_midnight = timestamp(utc_day_boundary - ONE_MINUTE_IN_MICROS);
        let after_utc_midnight = timestamp(utc_day_boundary + ONE_MINUTE_IN_MICROS);

        // Without offset these are different UTC days, so claimable
        assert!(can_claim_daily_reward(
            Some(&before_utc_midnight),
            after_utc_midnight,
            0
        ));

        // With +2h offset both timestamps fall on the same local day
        assert!(!can_claim_daily_reward(
            Some(&before_utc_midnight),
            after_utc_midnight,
            120
        ));
    }

    #[test]
    fn day_8_shows_only_current_day() {
        let result = build_daily_rewards(7, false);

        assert_eq!(
            result,
            DailyRewards {
                daily_reward_widgets_shown: vec![DailyRewardWidget { day: 8, gems: 15 }],
                show_three_points_before_last_daily_reward_widget: false,
                can_claim_first_daily_reward_widget: false,
            }
        );
    }
}
