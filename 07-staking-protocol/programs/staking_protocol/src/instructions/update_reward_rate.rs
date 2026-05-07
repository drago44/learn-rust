use crate::error::StakingError;
use crate::events::RewardRateUpdatedEvent;
use crate::helpers::update_reward_per_token;
use crate::state::StakingPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateRewardRate<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.stake_mint.as_ref()],
        bump = pool.bump,
        // Подвійний захист: лише `authority` пулу може змінювати ставку.
        has_one = authority @ StakingError::Unauthorized,
    )]
    pub pool: Account<'info, StakingPool>,
}

pub fn update_reward_rate_handler(ctx: Context<UpdateRewardRate>, new_rate: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // КРИТИЧНО: фіксуємо все нараховане за СТАРОЮ ставкою перед зміною rate.
    // Інакше rewards за минулий період були б ретроактивно перерахованими за новою ставкою.
    update_reward_per_token(pool)?;

    let old_rate = pool.reward_rate;
    pool.reward_rate = new_rate;

    emit!(RewardRateUpdatedEvent {
        authority: ctx.accounts.authority.key(),
        pool: pool.key(),
        old_rate,
        new_rate,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
