use anchor_lang::prelude::*;

/// Емітується після успішного `stake`. Off-chain індексатори можуть
/// парсити логи транзакції і відновлювати TVL пулу та позиції користувачів.
#[event]
pub struct StakedEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub user_total_staked: u64,
    pub pool_total_staked: u64,
    pub timestamp: i64,
}

#[event]
pub struct UnstakeRequestedEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub unstake_request: Pubkey,
    pub amount: u64,
    pub user_remaining_staked: u64,
    pub pool_total_staked: u64,
    pub created_at: i64,
}

#[event]
pub struct ClaimedEvent {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub unstake_request: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct HarvestedEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardRateUpdatedEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub old_rate: u64,
    pub new_rate: u64,
    pub timestamp: i64,
}

#[event]
pub struct PoolInitializedEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_rate: u64,
    pub timestamp: i64,
}
