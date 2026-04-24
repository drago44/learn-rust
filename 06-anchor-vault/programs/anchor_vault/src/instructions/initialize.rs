use anchor_lang::prelude::*;

use crate::state::VaultState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = VaultState::INIT_SPACE,
        seeds = [b"state", owner.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.vault_state.vault_bump = ctx.bumps.vault;
    ctx.accounts.vault_state.state_bump = ctx.bumps.vault_state;
    Ok(())
}
