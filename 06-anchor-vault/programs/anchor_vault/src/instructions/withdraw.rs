use anchor_lang::{prelude::*, system_program};

use crate::state::VaultState;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"state", owner.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn withdraw_handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let owner_key = ctx.accounts.owner.key();
    let seeds = &[
        b"vault",
        owner_key.as_ref(),
        &[ctx.accounts.vault_state.vault_bump],
    ];
    let signer_seeds = &[seeds.as_slice()];

    let cpi_accounts = system_program::Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        cpi_accounts,
        signer_seeds,
    );
    system_program::transfer(cpi_ctx, amount)
}
