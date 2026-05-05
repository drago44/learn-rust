use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Cooldown period has not expired")]
    CooldownNotExpired,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid mint")]
    InvalidMint,
}
