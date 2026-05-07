use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

/// Скільки секунд має пройти між `unstake` (створенням `UnstakeRequest`) і
/// `claim` (виводом токенів з vault до юзера). 7 днів.
pub const COOLDOWN_SECONDS: i64 = 7 * 24 * 60 * 60;
