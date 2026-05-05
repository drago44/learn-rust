pub mod constants;
pub mod error;
pub mod helpers;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("GE8xMAqBsFo2aJTEze8JR3BK99DuWWRN47tuncGPeTMM");

#[program]
pub mod staking_protocol {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
        initialize::initialize_pool_handler(ctx, reward_rate)
    }
}
