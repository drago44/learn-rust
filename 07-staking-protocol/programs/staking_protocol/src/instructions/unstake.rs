use crate::error::StakingError;
use crate::helpers::update_reward_per_token;
use crate::state::{StakingPool, UnstakeRequest, UserStake};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.stake_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"user", pool.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump,
    )]
    pub user_stake: Account<'info, UserStake>,

    // Новий акаунт-квиток на вивід.
    // Seeds містять timestamp — тому юзер може мати кілька паралельних запитів.
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 8 + 1,
        seeds = [b"unstake", user.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump,
    )]
    pub unstake_request: Account<'info, UnstakeRequest>,

    pub system_program: Program<'info, System>,
}

pub fn unstake_handler(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);
    require!(
        ctx.accounts.user_stake.amount_staked >= amount,
        StakingError::InsufficientStake
    );

    let pool = &mut ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;
    let unstake_request = &mut ctx.accounts.unstake_request;
    let now = Clock::get()?.unix_timestamp;

    // Оновлюємо індекс ПЕРЕД зміною балансів
    update_reward_per_token(pool)?;

    // Зменшуємо баланси — токени ще у vault, але вже "заброньовані" на вивід
    pool.total_staked = pool
        .total_staked
        .checked_sub(amount)
        .ok_or(StakingError::MathOverflow)?;

    user_stake.amount_staked = user_stake
        .amount_staked
        .checked_sub(amount)
        .ok_or(StakingError::MathOverflow)?;

    // Записуємо квиток — claim перевірить його через 7 днів
    unstake_request.owner = ctx.accounts.user.key();
    unstake_request.amount = amount;
    unstake_request.request_time = now;
    unstake_request.bump = ctx.bumps.unstake_request;

    Ok(())
}
