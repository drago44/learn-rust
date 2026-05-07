use crate::constants::COOLDOWN_SECONDS;
use crate::error::StakingError;
use crate::events::ClaimedEvent;
use crate::state::{StakingPool, UnstakeRequest};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Claim<'info> {
    /// Перейменовано на `owner`, щоб одразу попасти в `has_one = owner`
    /// перевірку проти `unstake_request.owner`.
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"pool", pool.stake_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, StakingPool>,

    #[account(
        mut,
        // Подвійний захист: seeds зав'язані на `owner` + `has_one` — на поле `owner`.
        seeds = [
            b"unstake",
            pool.key().as_ref(),
            owner.key().as_ref(),
            &unstake_request.request_time.to_le_bytes(),
        ],
        bump = unstake_request.bump,
        has_one = owner @ StakingError::Unauthorized,
        // Закриваємо акаунт після claim — rent повертається owner-у.
        close = owner,
    )]
    pub unstake_request: Account<'info, UnstakeRequest>,

    /// Куди повертаємо принципал (стейк-токени).
    #[account(
        mut,
        token::mint = pool.stake_mint,
        token::authority = owner,
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
}

pub fn claim_handler(ctx: Context<Claim>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let request = &ctx.accounts.unstake_request;

    let unlock_at = request
        .created_at
        .checked_add(COOLDOWN_SECONDS)
        .ok_or(StakingError::MathOverflow)?;
    require!(now >= unlock_at, StakingError::CooldownNotExpired);

    let amount = request.amount;

    // CPI: vault (authority = pool PDA) → user_token_account
    let pool = &ctx.accounts.pool;
    let stake_mint_key = pool.stake_mint;
    let pool_bump = pool.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", stake_mint_key.as_ref(), &[pool_bump]]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
        ctx.accounts.stake_mint.decimals,
    )?;

    emit!(ClaimedEvent {
        owner: ctx.accounts.owner.key(),
        pool: ctx.accounts.pool.key(),
        unstake_request: ctx.accounts.unstake_request.key(),
        amount,
        timestamp: now,
    });

    // Закриття акаунта (rent → owner) робить Anchor через `close = owner`.
    Ok(())
}
