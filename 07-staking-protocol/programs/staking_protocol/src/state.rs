use anchor_lang::prelude::*;

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub vault: Pubkey,        // зберігає застейкані токени (stake_mint)
    pub reward_vault: Pubkey, // зберігає reward токени (reward_mint), authority = pool
    pub total_staked: u64,
    pub reward_rate: u64,
    pub reward_per_token_stored: u128,
    pub last_update_time: i64,
    pub bump: u8,
}

impl StakingPool {
    // 8 disc + 5 * Pubkey + u64 + u64 + u128 + i64 + u8
    pub const SIZE: usize = 8 + 32 * 5 + 8 + 8 + 16 + 8 + 1;
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub amount_staked: u64,
    pub reward_debt: u128, // reward_per_token_stored на момент останньої акумуляції
    pub pending_rewards: u64, // нараховані, але ще не вилучені rewards
    pub bump: u8,
}

impl UserStake {
    // 8 disc + Pubkey + u64 + u128 + u64 + u8
    pub const SIZE: usize = 8 + 32 + 8 + 16 + 8 + 1;
}

#[account]
pub struct UnstakeRequest {
    pub owner: Pubkey,     // хто створив запит
    pub amount: u64,       // скільки токенів виводять
    pub request_time: i64, // nonce з параметра клієнта — використовується тільки в seeds для унікальності PDA
    pub created_at: i64,   // реальний on-chain Clock::get() — від нього тікає COOLDOWN_SECONDS
    pub bump: u8,
}

impl UnstakeRequest {
    // 8 disc + Pubkey + u64 + i64 + i64 + u8
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1;
}
