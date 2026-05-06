use anchor_lang::prelude::*;

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub vault: Pubkey,
    pub total_staked: u64,
    pub reward_rate: u64,
    pub reward_per_token_stored: u128,
    pub last_update_time: i64,
    pub bump: u8,
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub amount_staked: u64,
    pub reward_debt: u128,
    pub bump: u8,
}

#[account]
pub struct UnstakeRequest {
    pub owner: Pubkey,     // хто створив запит
    pub amount: u64,       // скільки токенів виводять
    pub request_time: i64, // unix timestamp коли створено
    pub bump: u8,
}
