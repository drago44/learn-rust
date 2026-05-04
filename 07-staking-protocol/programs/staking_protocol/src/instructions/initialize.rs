use crate::state::StakingPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 16 + 8 + 1,
        seeds = [b"pool", stake_mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, StakingPool>,

    pub stake_mint: InterfaceAccount<'info, anchor_spl::token_interface::Mint>,
    pub reward_mint: InterfaceAccount<'info, anchor_spl::token_interface::Mint>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_pool_handler(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.authority = ctx.accounts.authority.key();
    pool.stake_mint = ctx.accounts.stake_mint.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.total_staked = 0;
    pool.reward_rate = reward_rate;
    pool.reward_per_token_stored = 0;
    pool.last_update_time = Clock::get()?.unix_timestamp;
    pool.bump = ctx.bumps.pool;

    Ok(())
}
