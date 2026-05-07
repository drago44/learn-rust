pub mod constants;
pub mod error;
pub mod events;
pub mod helpers;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("8TDLJ18auzqhoQTFNsnijVSYBJNxxWqhdAvFw2shqsud");

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

    pub fn update_reward_rate(ctx: Context<UpdateRewardRate>, new_rate: u64) -> Result<()> {
        update_reward_rate::update_reward_rate_handler(ctx, new_rate)
    }
}
