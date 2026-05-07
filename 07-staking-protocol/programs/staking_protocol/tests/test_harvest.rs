use anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_sdk::clock::Clock;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spl_token::state::Mint;

// === PDAs ===

fn pool_pda(stake_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool", stake_mint.as_ref()], program_id)
}

fn vault_pda(pool: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", pool.as_ref()], program_id)
}

fn reward_vault_pda(pool: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"reward_vault", pool.as_ref()], program_id)
}

fn user_stake_pda(pool: &Pubkey, user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"user", pool.as_ref(), user.as_ref()], program_id)
}

// === Tx helpers ===

fn send(
    svm: &mut LiteSVM,
    ix: Instruction,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&signers[0].pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn create_mint(svm: &mut LiteSVM, payer: &Keypair, decimals: u8) -> Pubkey {
    let mint = Keypair::new();
    let rent = svm.minimum_balance_for_rent_exemption(Mint::LEN);
    let ixs = [
        solana_system_interface::instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            rent,
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            &payer.pubkey(),
            None,
            decimals,
        )
        .unwrap(),
    ];
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer, &mint]).unwrap();
    svm.send_transaction(tx).unwrap();
    mint.pubkey()
}

fn create_token_account(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Pubkey {
    let token_account = Keypair::new();
    let rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Account::LEN);
    let ixs = [
        solana_system_interface::instruction::create_account(
            &payer.pubkey(),
            &token_account.pubkey(),
            rent,
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_account3(
            &spl_token::id(),
            &token_account.pubkey(),
            mint,
            owner,
        )
        .unwrap(),
    ];
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer, &token_account])
        .unwrap();
    svm.send_transaction(tx).unwrap();
    token_account.pubkey()
}

fn mint_to(svm: &mut LiteSVM, mint_authority: &Keypair, mint: &Pubkey, dest: &Pubkey, amount: u64) {
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        dest,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    send(svm, ix, &[mint_authority]).unwrap();
}

fn create_token_account_with_tokens(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
    amount: u64,
) -> Pubkey {
    let token_account = create_token_account(svm, payer, mint, owner);
    mint_to(svm, payer, mint, &token_account, amount);
    token_account
}

/// Просуває on-chain clock на `secs` секунд (LiteSVM clock сам не йде).
fn warp_clock_secs(svm: &mut LiteSVM, secs: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp += secs;
    svm.set_sysvar(&clock);
}

fn token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
    let acc = svm.get_account(account).unwrap();
    spl_token::state::Account::unpack(&acc.data).unwrap().amount
}

// === Pool / instruction helpers ===

struct PoolCtx {
    program_id: Pubkey,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    pool: Pubkey,
    vault: Pubkey,
    reward_vault: Pubkey,
}

fn load_program(svm: &mut LiteSVM) {
    let bytes = include_bytes!("../../../target/deploy/staking_protocol.so");
    svm.add_program(staking_protocol::id(), bytes).unwrap();
}

fn setup_pool(
    svm: &mut LiteSVM,
    authority: &Keypair,
    reward_rate: u64,
    reward_supply: u64,
) -> PoolCtx {
    let program_id = staking_protocol::id();
    let stake_mint = create_mint(svm, authority, 6);
    let reward_mint = create_mint(svm, authority, 6);
    let (pool, _) = pool_pda(&stake_mint, &program_id);
    let (vault, _) = vault_pda(&pool, &program_id);
    let (reward_vault, _) = reward_vault_pda(&pool, &program_id);

    let ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::InitializePool { reward_rate }.data(),
        staking_protocol::accounts::InitializePool {
            authority: authority.pubkey(),
            pool,
            vault,
            reward_vault,
            stake_mint,
            reward_mint,
            token_program: spl_token::id(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[authority]).unwrap();

    // Адмін як mint_authority наповнює reward_vault
    if reward_supply > 0 {
        mint_to(svm, authority, &reward_mint, &reward_vault, reward_supply);
    }

    PoolCtx {
        program_id,
        stake_mint,
        reward_mint,
        pool,
        vault,
        reward_vault,
    }
}

fn do_stake(
    svm: &mut LiteSVM,
    ctx: &PoolCtx,
    user: &Keypair,
    user_token_account: Pubkey,
    amount: u64,
) {
    let (user_stake, _) = user_stake_pda(&ctx.pool, &user.pubkey(), &ctx.program_id);
    let ix = Instruction::new_with_bytes(
        ctx.program_id,
        &staking_protocol::instruction::Stake { amount }.data(),
        staking_protocol::accounts::Stake {
            user: user.pubkey(),
            pool: ctx.pool,
            user_stake,
            user_token_account,
            vault: ctx.vault,
            stake_mint: ctx.stake_mint,
            token_program: spl_token::id(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[user]).unwrap();
}

fn do_harvest(
    svm: &mut LiteSVM,
    ctx: &PoolCtx,
    user: &Keypair,
    user_reward_account: Pubkey,
) -> litesvm::types::TransactionResult {
    let (user_stake, _) = user_stake_pda(&ctx.pool, &user.pubkey(), &ctx.program_id);
    let ix = Instruction::new_with_bytes(
        ctx.program_id,
        &staking_protocol::instruction::Harvest {}.data(),
        staking_protocol::accounts::Harvest {
            user: user.pubkey(),
            pool: ctx.pool,
            user_stake,
            user_reward_account,
            reward_vault: ctx.reward_vault,
            reward_mint: ctx.reward_mint,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[user])
}

// === Tests ===

/// Одразу після стейку — нічого не нараховано.
#[test]
fn test_harvest_zero_immediately_after_stake() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let ctx = setup_pool(&mut svm, &authority, 1_000, 1_000_000_000);
    let user_token = create_token_account_with_tokens(
        &mut svm,
        &authority,
        &ctx.stake_mint,
        &user.pubkey(),
        1_000_000,
    );
    let user_reward = create_token_account(&mut svm, &authority, &ctx.reward_mint, &user.pubkey());

    do_stake(&mut svm, &ctx, &user, user_token, 500_000);
    do_harvest(&mut svm, &ctx, &user, user_reward).unwrap();

    assert_eq!(token_balance(&svm, &user_reward), 0);
}

/// Просуваємо clock — юзер забирає рівно `elapsed × reward_rate` (бо стейкає сам).
#[test]
fn test_harvest_after_time_elapsed() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let reward_rate = 1_000u64;
    let ctx = setup_pool(&mut svm, &authority, reward_rate, 10_000_000_000);
    let user_token = create_token_account_with_tokens(
        &mut svm,
        &authority,
        &ctx.stake_mint,
        &user.pubkey(),
        1_000_000,
    );
    let user_reward = create_token_account(&mut svm, &authority, &ctx.reward_mint, &user.pubkey());

    do_stake(&mut svm, &ctx, &user, user_token, 1_000_000);

    // 100 секунд × 1000 reward/s = 100_000 (юзер володіє 100% пулу)
    warp_clock_secs(&mut svm, 100);
    do_harvest(&mut svm, &ctx, &user, user_reward).unwrap();

    assert_eq!(token_balance(&svm, &user_reward), 100_000);
}

/// Harvest двічі підряд — другий раз 0 (нічого нового не нараховано).
#[test]
fn test_harvest_twice_second_is_zero() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let ctx = setup_pool(&mut svm, &authority, 1_000, 10_000_000_000);
    let user_token = create_token_account_with_tokens(
        &mut svm,
        &authority,
        &ctx.stake_mint,
        &user.pubkey(),
        1_000_000,
    );
    let user_reward = create_token_account(&mut svm, &authority, &ctx.reward_mint, &user.pubkey());

    do_stake(&mut svm, &ctx, &user, user_token, 1_000_000);
    warp_clock_secs(&mut svm, 100);

    do_harvest(&mut svm, &ctx, &user, user_reward).unwrap();
    let first = token_balance(&svm, &user_reward);
    assert_eq!(first, 100_000);

    // Між двома harvest clock не рухається — pending має бути 0.
    // Лише expire-аємо blockhash щоб LiteSVM не відхилив транзу як AlreadyProcessed
    // (та сама інструкція + той самий blockhash → ідентичний signature).
    svm.expire_blockhash();
    do_harvest(&mut svm, &ctx, &user, user_reward).unwrap();
    let second = token_balance(&svm, &user_reward);
    assert_eq!(second, first, "другий harvest не повинен додати rewards");
}

/// Два стейкери — rewards розподіляються пропорційно `amount_staked`.
#[test]
fn test_harvest_two_stakers_proportional() {
    let authority = Keypair::new();
    let user1 = Keypair::new();
    let user2 = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user1.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user2.pubkey(), 10_000_000_000).unwrap();

    let ctx = setup_pool(&mut svm, &authority, 1_000, 10_000_000_000);

    let token1 = create_token_account_with_tokens(
        &mut svm,
        &authority,
        &ctx.stake_mint,
        &user1.pubkey(),
        600_000,
    );
    let token2 = create_token_account_with_tokens(
        &mut svm,
        &authority,
        &ctx.stake_mint,
        &user2.pubkey(),
        400_000,
    );
    let reward1 = create_token_account(&mut svm, &authority, &ctx.reward_mint, &user1.pubkey());
    let reward2 = create_token_account(&mut svm, &authority, &ctx.reward_mint, &user2.pubkey());

    // Обидва стейкають у "одну мить" (clock у LiteSVM не рухається сам).
    do_stake(&mut svm, &ctx, &user1, token1, 600_000);
    do_stake(&mut svm, &ctx, &user2, token2, 400_000);

    // 100с × 1000 = 100_000 на пул, юзер1 забере 60%, юзер2 — 40%.
    warp_clock_secs(&mut svm, 100);

    do_harvest(&mut svm, &ctx, &user1, reward1).unwrap();
    do_harvest(&mut svm, &ctx, &user2, reward2).unwrap();

    assert_eq!(token_balance(&svm, &reward1), 60_000);
    assert_eq!(token_balance(&svm, &reward2), 40_000);
}

/// total_staked = 0 + проходить час → перший stake не повинен впасти від ділення на нуль.
#[test]
fn test_no_division_by_zero_when_pool_is_empty() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let ctx = setup_pool(&mut svm, &authority, 1_000, 10_000_000_000);
    let user_token = create_token_account_with_tokens(
        &mut svm,
        &authority,
        &ctx.stake_mint,
        &user.pubkey(),
        500_000,
    );

    // Час іде поки в пулі ніхто не стейкає
    warp_clock_secs(&mut svm, 1_000);

    // Перший stake — update_reward_per_token має побачити total_staked = 0 і не ділити.
    do_stake(&mut svm, &ctx, &user, user_token, 500_000);

    // Перевіряємо що пул дійшов сюди і amount записався
    let pool_account = svm.get_account(&ctx.pool).unwrap();
    let pool_data: staking_protocol::state::StakingPool =
        anchor_lang::AccountDeserialize::try_deserialize(&mut pool_account.data.as_slice())
            .unwrap();
    assert_eq!(pool_data.total_staked, 500_000);
    // reward_per_token_stored залишився 0 — нічого нараховувати поки пул був порожній.
    assert_eq!(pool_data.reward_per_token_stored, 0);
}
