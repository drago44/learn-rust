use crate::error::StakingError;
use crate::events::UnstakeRequestedEvent;
use crate::helpers::accrue_user_rewards;
use crate::state::{StakingPool, UnstakeRequest, UserStake};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(amount: u64, request_time: i64)]
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

    // Кожен unstake — окремий акаунт з унікальним timestamp.
    // Можна мати кілька паралельних запитів одночасно.
    #[account(
        init,
        payer = user,
        space = UnstakeRequest::SIZE,
        seeds = [b"unstake", pool.key().as_ref(), user.key().as_ref(), &request_time.to_le_bytes()],
        bump,
    )]
    pub unstake_request: Account<'info, UnstakeRequest>,

    pub system_program: Program<'info, System>,
}

pub fn unstake_handler(ctx: Context<Unstake>, amount: u64, request_time: i64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);
    require!(
        ctx.accounts.user_stake.amount_staked >= amount,
        StakingError::InsufficientStake
    );

    let pool = &mut ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;
    let unstake_request = &mut ctx.accounts.unstake_request;

    // Settle rewards перед зменшенням amount_staked, щоб не втратити нараховане.
    accrue_user_rewards(pool, user_stake)?;

    // Зменшуємо баланси — токени ще у vault, але заброньовані на вивід
    pool.total_staked = pool
        .total_staked
        .checked_sub(amount)
        .ok_or(StakingError::MathOverflow)?;

    user_stake.amount_staked = user_stake
        .amount_staked
        .checked_sub(amount)
        .ok_or(StakingError::MathOverflow)?;

    // Записуємо квиток.
    // request_time = параметр (nonce/seed) — щоб claim міг ре-дерайвнути той самий PDA.
    // created_at = реальний on-chain час — від нього рахуємо cooldown (атакер не впливає).
    unstake_request.owner = ctx.accounts.user.key();
    unstake_request.amount = amount;
    unstake_request.request_time = request_time;
    unstake_request.created_at = Clock::get()?.unix_timestamp;
    unstake_request.bump = ctx.bumps.unstake_request;

    emit!(UnstakeRequestedEvent {
        user: ctx.accounts.user.key(),
        pool: pool.key(),
        unstake_request: unstake_request.key(),
        amount,
        user_remaining_staked: user_stake.amount_staked,
        pool_total_staked: pool.total_staked,
        created_at: unstake_request.created_at,
    });

    Ok(())
}
