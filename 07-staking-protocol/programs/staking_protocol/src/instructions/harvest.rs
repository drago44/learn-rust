use crate::error::StakingError;
use crate::helpers::accrue_user_rewards;
use crate::state::{StakingPool, UserStake};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Harvest<'info> {
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

    #[account(
        mut,
        token::mint = pool.reward_mint,
        token::authority = user,
        token::token_program = token_program,
    )]
    pub user_reward_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.reward_vault,
        token::token_program = token_program,
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(address = pool.reward_mint)]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn harvest_handler(ctx: Context<Harvest>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;

    // Захист на випадок зіпсованого user_stake (перевірка зайва, бо seeds збігаються
    // тільки якщо user_stake належить цьому юзеру, але явність дешева).
    require_keys_eq!(
        user_stake.owner,
        ctx.accounts.user.key(),
        StakingError::Unauthorized
    );

    // Settle: фіксуємо нараховане у user_stake.pending_rewards.
    accrue_user_rewards(pool, user_stake)?;

    let amount = user_stake.pending_rewards;
    if amount == 0 {
        // Нічого виводити — повертаємо Ok щоб не палити транзу.
        return Ok(());
    }

    // Обнуляємо ДО CPI (re-entrancy hygiene; у Solana менш критично, але паттерн коректний).
    user_stake.pending_rewards = 0;

    // CPI: переводимо reward токени з reward_vault до юзера.
    // Authority = pool PDA, тож підписуємо seeds-ами пулу.
    let stake_mint_key = pool.stake_mint;
    let pool_bump = pool.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", stake_mint_key.as_ref(), &[pool_bump]]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.reward_vault.to_account_info(),
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx.accounts.user_reward_account.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
        ctx.accounts.reward_mint.decimals,
    )?;

    Ok(())
}
