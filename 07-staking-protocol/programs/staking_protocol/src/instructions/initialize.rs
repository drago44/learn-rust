use crate::events::PoolInitializedEvent;
use crate::state::StakingPool;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = StakingPool::SIZE,
        seeds = [b"pool", stake_mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, StakingPool>,

    // vault зберігає застейкані токени; authority = pool PDA
    #[account(
        init,
        payer = authority,
        token::mint = stake_mint,
        token::authority = pool,
        token::token_program = token_program,
        seeds = [b"vault", pool.key().as_ref()],
        bump,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // reward_vault — окремий vault для reward токенів; authority = pool PDA.
    // Адмін наповнює його окремо (mint_to / transfer) після ініціалізації.
    #[account(
        init,
        payer = authority,
        token::mint = reward_mint,
        token::authority = pool,
        token::token_program = token_program,
        seeds = [b"reward_vault", pool.key().as_ref()],
        bump,
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    pub stake_mint: InterfaceAccount<'info, Mint>,
    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_pool_handler(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.authority = ctx.accounts.authority.key();
    pool.stake_mint = ctx.accounts.stake_mint.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.vault = ctx.accounts.vault.key();
    pool.reward_vault = ctx.accounts.reward_vault.key();
    pool.total_staked = 0;
    pool.reward_rate = reward_rate;
    pool.reward_per_token_stored = 0;
    pool.last_update_time = Clock::get()?.unix_timestamp;
    pool.bump = ctx.bumps.pool;

    emit!(PoolInitializedEvent {
        authority: pool.authority,
        pool: pool.key(),
        stake_mint: pool.stake_mint,
        reward_mint: pool.reward_mint,
        reward_rate,
        timestamp: pool.last_update_time,
    });

    Ok(())
}
