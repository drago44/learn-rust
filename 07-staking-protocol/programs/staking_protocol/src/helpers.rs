
use crate::error::StakingError;
use crate::state::StakingPool;
use anchor_lang::prelude::*;

pub fn update_reward_per_token(pool: &mut StakingPool) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    if pool.total_staked > 0 {
        let elapsed = (now - pool.last_update_time) as u128;

        let delta = elapsed
            .checked_mul(pool.reward_rate as u128)
            .and_then(|v| v.checked_mul(1_000_000_000))
            .and_then(|v| v.checked_div(pool.total_staked as u128))
            .ok_or(StakingError::MathOverflow)?;

        pool.reward_per_token_stored = pool
            .reward_per_token_stored
            .checked_add(delta)
            .ok_or(StakingError::MathOverflow)?;
    }

    pool.last_update_time = now;
    Ok(())
}
