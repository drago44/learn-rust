pub mod constants;
pub mod error;
pub mod helpers;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("FZ24TszdtKWgUGsBV3TiTNni3XzXMrde8JDK5eGtTP9j");

#[program]
pub mod staking_protocol {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
        initialize::initialize_pool_handler(ctx, reward_rate)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        stake::stake_handler(ctx, amount)
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64, request_time: i64) -> Result<()> {
        unstake::unstake_handler(ctx, amount, request_time)
    }

    pub fn harvest(ctx: Context<Harvest>) -> Result<()> {
        harvest::harvest_handler(ctx)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        claim::claim_handler(ctx)
    }
}
