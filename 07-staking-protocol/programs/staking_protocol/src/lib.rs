pub mod constants;
pub mod error;
pub mod helpers;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("7SE4VwNUteR1JdVtDtNRMNpYciDj7zUR4y7EZpm9npnh");

#[program]
pub mod staking_protocol {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
        initialize::initialize_pool_handler(ctx, reward_rate)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        stake::stake_handler(ctx, amount)
    }
}
