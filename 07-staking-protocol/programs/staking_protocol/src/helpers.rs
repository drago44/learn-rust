use crate::error::StakingError;
use crate::state::{StakingPool, UserStake};
use anchor_lang::prelude::*;

/// Множник точності для reward_per_token (щоб уникнути втрати дробових частин).
pub const PRECISION: u128 = 1_000_000_000;

/// Оновлює глобальний індекс reward_per_token_stored на основі часу, що пройшов.
/// Викликається ПЕРЕД будь-якою зміною balance юзера або pool.total_staked.
pub fn update_reward_per_token(pool: &mut StakingPool) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    if pool.total_staked > 0 {
        let elapsed = now
            .checked_sub(pool.last_update_time)
            .ok_or(StakingError::MathOverflow)? as u128;

        let delta = elapsed
            .checked_mul(pool.reward_rate as u128)
            .and_then(|v| v.checked_mul(PRECISION))
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

/// Скільки rewards "заробив" юзер з моменту останньої акумуляції,
/// беручи поточний індекс пулу як точку відліку.
pub fn earned(pool: &StakingPool, user: &UserStake) -> Result<u64> {
    // delta — наскільки індекс зріс з моменту user.reward_debt
    let delta = pool
        .reward_per_token_stored
        .checked_sub(user.reward_debt)
        .ok_or(StakingError::MathOverflow)?;

    let reward = delta
        .checked_mul(user.amount_staked as u128)
        .and_then(|v| v.checked_div(PRECISION))
        .ok_or(StakingError::MathOverflow)?;

    u64::try_from(reward).map_err(|_| StakingError::MathOverflow.into())
}

/// Synthetix-style "settlement": перш ніж міняти amount_staked,
/// фіксуємо все що юзер заробив, і ставимо reward_debt на поточний індекс.
/// Після цього майбутні зміни amount_staked не зіпсують вже нараховане.
pub fn accrue_user_rewards(pool: &mut StakingPool, user: &mut UserStake) -> Result<()> {
    update_reward_per_token(pool)?;

    let new_pending = earned(pool, user)?;
    user.pending_rewards = user
        .pending_rewards
        .checked_add(new_pending)
        .ok_or(StakingError::MathOverflow)?;

    user.reward_debt = pool.reward_per_token_stored;
    Ok(())
}
