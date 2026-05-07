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

const COOLDOWN_SECONDS: i64 = 7 * 24 * 60 * 60;

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

fn unstake_request_pda(
    pool: &Pubkey,
    user: &Pubkey,
    request_time: i64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"unstake",
            pool.as_ref(),
            user.as_ref(),
            &request_time.to_le_bytes(),
        ],
        program_id,
    )
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

fn warp_clock_secs(svm: &mut LiteSVM, secs: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp += secs;
    svm.set_sysvar(&clock);
}

fn token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
    let acc = svm.get_account(account).unwrap();
    spl_token::state::Account::unpack(&acc.data).unwrap().amount
}

// === Setup ===

struct PoolCtx {
    program_id: Pubkey,
    stake_mint: Pubkey,
    pool: Pubkey,
    vault: Pubkey,
}

fn load_program(svm: &mut LiteSVM) {
    let bytes = include_bytes!("../../../target/deploy/staking_protocol.so");
    svm.add_program(staking_protocol::id(), bytes).unwrap();
}

/// Створює пул, стейкає `stake_amount` від імені `user`, повертає
/// контекст і `user_token_account` (буде використаний як destination для claim).
fn setup_with_stake(
    svm: &mut LiteSVM,
    authority: &Keypair,
    user: &Keypair,
    stake_amount: u64,
) -> (PoolCtx, Pubkey) {
    let program_id = staking_protocol::id();
    let stake_mint = create_mint(svm, authority, 6);
    let reward_mint = create_mint(svm, authority, 6);
    let (pool, _) = pool_pda(&stake_mint, &program_id);
    let (vault, _) = vault_pda(&pool, &program_id);
    let (reward_vault, _) = reward_vault_pda(&pool, &program_id);

    let init_ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::InitializePool { reward_rate: 1_000 }.data(),
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
    send(svm, init_ix, &[authority]).unwrap();

    let user_token_account =
        create_token_account_with_tokens(svm, authority, &stake_mint, &user.pubkey(), stake_amount);
    let (user_stake, _) = user_stake_pda(&pool, &user.pubkey(), &program_id);

    let stake_ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::Stake {
            amount: stake_amount,
        }
        .data(),
        staking_protocol::accounts::Stake {
            user: user.pubkey(),
            pool,
            user_stake,
            user_token_account,
            vault,
            stake_mint,
            token_program: spl_token::id(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send(svm, stake_ix, &[user]).unwrap();

    (
        PoolCtx {
            program_id,
            stake_mint,
            pool,
            vault,
        },
        user_token_account,
    )
}

fn do_unstake(svm: &mut LiteSVM, ctx: &PoolCtx, user: &Keypair, amount: u64, request_time: i64) {
    let (user_stake, _) = user_stake_pda(&ctx.pool, &user.pubkey(), &ctx.program_id);
    let (unstake_request, _) =
        unstake_request_pda(&ctx.pool, &user.pubkey(), request_time, &ctx.program_id);

    let ix = Instruction::new_with_bytes(
        ctx.program_id,
        &staking_protocol::instruction::Unstake {
            amount,
            request_time,
        }
        .data(),
        staking_protocol::accounts::Unstake {
            user: user.pubkey(),
            pool: ctx.pool,
            user_stake,
            unstake_request,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[user]).unwrap();
}

fn do_claim(
    svm: &mut LiteSVM,
    ctx: &PoolCtx,
    owner: &Keypair,
    user_token_account: Pubkey,
    unstake_request: Pubkey,
) -> litesvm::types::TransactionResult {
    let ix = Instruction::new_with_bytes(
        ctx.program_id,
        &staking_protocol::instruction::Claim {}.data(),
        staking_protocol::accounts::Claim {
            owner: owner.pubkey(),
            pool: ctx.pool,
            unstake_request,
            user_token_account,
            vault: ctx.vault,
            stake_mint: ctx.stake_mint,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[owner])
}

// === Tests ===

/// `claim` до закінчення cooldown — помилка `CooldownNotExpired`.
#[test]
fn test_claim_before_cooldown_fails() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let (ctx, user_token) = setup_with_stake(&mut svm, &authority, &user, 1_000_000);

    let request_time = 1_000_000i64;
    do_unstake(&mut svm, &ctx, &user, 400_000, request_time);

    let (req, _) = unstake_request_pda(&ctx.pool, &user.pubkey(), request_time, &ctx.program_id);

    // Просунули час, але менше ніж 7 днів
    warp_clock_secs(&mut svm, COOLDOWN_SECONDS - 60);

    let res = do_claim(&mut svm, &ctx, &user, user_token, req);
    assert!(res.is_err(), "claim до cooldown мав би зафейлитись");
}

/// `claim` після cooldown — токени повертаються, акаунт `UnstakeRequest` закривається.
#[test]
fn test_claim_after_cooldown_returns_tokens_and_closes_account() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let (ctx, user_token) = setup_with_stake(&mut svm, &authority, &user, 1_000_000);

    // Після стейку у юзера 0 токенів (всі у vault)
    assert_eq!(token_balance(&svm, &user_token), 0);
    assert_eq!(token_balance(&svm, &ctx.vault), 1_000_000);

    let request_time = 1_000_000i64;
    do_unstake(&mut svm, &ctx, &user, 400_000, request_time);

    let (req, _) = unstake_request_pda(&ctx.pool, &user.pubkey(), request_time, &ctx.program_id);
    // unstake_request існує
    assert!(svm.get_account(&req).is_some());

    // Стрибаємо на 7+ днів
    warp_clock_secs(&mut svm, COOLDOWN_SECONDS + 1);

    do_claim(&mut svm, &ctx, &user, user_token, req).unwrap();

    // Юзер забрав 400_000, у vault залишилось 600_000
    assert_eq!(token_balance(&svm, &user_token), 400_000);
    assert_eq!(token_balance(&svm, &ctx.vault), 600_000);

    // Акаунт закритий: Anchor `close = owner` обнуляє lamports і дані;
    // LiteSVM віддає `None` для повністю закритих акаунтів.
    let after = svm.get_account(&req);
    let closed = match &after {
        None => true,
        Some(acc) => acc.lamports == 0 && acc.data.is_empty(),
    };
    assert!(
        closed,
        "UnstakeRequest мав бути закритий, але існує: {:?}",
        after
    );
}

/// Чужий юзер пробує claim чужого `UnstakeRequest` — помилка `Unauthorized`
/// (фактично спрацює `seeds`-перевірка раніше, але результат — Err).
#[test]
fn test_claim_by_other_user_fails() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let attacker = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&attacker.pubkey(), 10_000_000_000).unwrap();

    let (ctx, _user_token) = setup_with_stake(&mut svm, &authority, &user, 1_000_000);

    let request_time = 1_000_000i64;
    do_unstake(&mut svm, &ctx, &user, 400_000, request_time);

    // Чужий створює свій token account для stake_mint, щоб claim не падав на
    // перевірці `token::authority = owner` ще до перевірки has_one/seeds.
    let attacker_token =
        create_token_account(&mut svm, &authority, &ctx.stake_mint, &attacker.pubkey());

    let (req, _) = unstake_request_pda(&ctx.pool, &user.pubkey(), request_time, &ctx.program_id);

    warp_clock_secs(&mut svm, COOLDOWN_SECONDS + 1);

    // Atacker пробує claim чужого `unstake_request` під своїм підписом
    let res = do_claim(&mut svm, &ctx, &attacker, attacker_token, req);
    assert!(res.is_err(), "чужий claim мав би зафейлитись");
}
