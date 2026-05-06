use crate::error::StakingError;
use crate::helpers::update_reward_per_token;
use crate::state::{StakingPool, UserStake};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.stake_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, StakingPool>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 16 + 1,
        seeds = [b"user", pool.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        token::mint = pool.stake_mint,
        token::authority = user,
        token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.vault,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(address = pool.stake_mint)]
    pub stake_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn stake_handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);

    let pool = &mut ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;

    // Оновлюємо індекс ПЕРЕД зміною балансів
    update_reward_per_token(pool)?;

    // Перший стейк цього юзера — ініціалізуємо поля
    if user_stake.owner == Pubkey::default() {
        user_stake.owner = ctx.accounts.user.key();
        user_stake.bump = ctx.bumps.user_stake;
    }

    // Фіксуємо поточний індекс — pending rewards = 0 одразу після стейку
    user_stake.reward_debt = pool.reward_per_token_stored;

    // CPI: перевести токени від юзера до vault
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.stake_mint.decimals,
    )?;

    pool.total_staked = pool
        .total_staked
        .checked_add(amount)
        .ok_or(StakingError::MathOverflow)?;

    user_stake.amount_staked = user_stake
        .amount_staked
        .checked_add(amount)
        .ok_or(StakingError::MathOverflow)?;

    Ok(())
}
