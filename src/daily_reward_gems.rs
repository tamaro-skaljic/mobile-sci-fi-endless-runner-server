use crate::constants::DAILY_REWARD_GEMS;

pub fn daily_reward_gems(number_of_claimed_daily_rewards: u16) -> u64 {
    let index = (number_of_claimed_daily_rewards as usize).min(DAILY_REWARD_GEMS.len() - 1);
    DAILY_REWARD_GEMS[index]
}
