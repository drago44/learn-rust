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

fn create_token_account_with_tokens(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
    amount: u64,
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
        spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            &token_account.pubkey(),
            &payer.pubkey(),
            &[],
            amount,
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

fn warp_clock_secs(svm: &mut LiteSVM, secs: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp += secs;
    svm.set_sysvar(&clock);
}

fn load_program(svm: &mut LiteSVM) {
    let bytes = include_bytes!("../../../target/deploy/staking_protocol.so");
    svm.add_program(staking_protocol::id(), bytes).unwrap();
}

fn load_pool(svm: &LiteSVM, pool: &Pubkey) -> staking_protocol::state::StakingPool {
    let acc = svm.get_account(pool).unwrap();
    anchor_lang::AccountDeserialize::try_deserialize(&mut acc.data.as_slice()).unwrap()
}

// === Setup ===

fn setup_pool(
    svm: &mut LiteSVM,
    authority: &Keypair,
    reward_rate: u64,
) -> (Pubkey, Pubkey, Pubkey) {
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
    (stake_mint, pool, vault)
}

fn do_stake(
    svm: &mut LiteSVM,
    pool: &Pubkey,
    user: &Keypair,
    user_token: Pubkey,
    vault: Pubkey,
    stake_mint: Pubkey,
    amount: u64,
) {
    let program_id = staking_protocol::id();
    let (user_stake, _) = user_stake_pda(pool, &user.pubkey(), &program_id);
    let ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::Stake { amount }.data(),
        staking_protocol::accounts::Stake {
            user: user.pubkey(),
            pool: *pool,
            user_stake,
            user_token_account: user_token,
            vault,
            stake_mint,
            token_program: spl_token::id(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[user]).unwrap();
}

fn do_update_rate(
    svm: &mut LiteSVM,
    pool: &Pubkey,
    authority: &Keypair,
    new_rate: u64,
) -> litesvm::types::TransactionResult {
    let program_id = staking_protocol::id();
    let ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::UpdateRewardRate { new_rate }.data(),
        staking_protocol::accounts::UpdateRewardRate {
            authority: authority.pubkey(),
            pool: *pool,
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[authority])
}

// === Tests ===

/// Адмін успішно змінює `reward_rate`.
#[test]
fn test_update_reward_rate_by_authority_ok() {
    let authority = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    let (_stake_mint, pool, _vault) = setup_pool(&mut svm, &authority, 1_000);
    assert_eq!(load_pool(&svm, &pool).reward_rate, 1_000);

    do_update_rate(&mut svm, &pool, &authority, 5_000).unwrap();
    assert_eq!(load_pool(&svm, &pool).reward_rate, 5_000);
}

/// Чужий юзер пробує змінити rate — `Unauthorized`.
#[test]
fn test_update_reward_rate_by_other_fails() {
    let authority = Keypair::new();
    let attacker = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&attacker.pubkey(), 10_000_000_000).unwrap();

    let (_stake_mint, pool, _vault) = setup_pool(&mut svm, &authority, 1_000);

    let res = do_update_rate(&mut svm, &pool, &attacker, 999_999);
    assert!(res.is_err(), "atacker не повинен мати доступу");
    assert_eq!(load_pool(&svm, &pool).reward_rate, 1_000);
}

/// Зміна rate "settle"-ить накопичений `reward_per_token_stored` за СТАРОЮ ставкою.
/// Інакше нараховане за минулий період було б ретроактивно перераховане за новою ставкою.
#[test]
fn test_update_reward_rate_preserves_accrued_rewards() {
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm);
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let (stake_mint, pool, vault) = setup_pool(&mut svm, &authority, 1_000);
    let user_token = create_token_account_with_tokens(
        &mut svm,
        &authority,
        &stake_mint,
        &user.pubkey(),
        1_000_000,
    );
    do_stake(
        &mut svm, &pool, &user, user_token, vault, stake_mint, 1_000_000,
    );

    // 100с при rate=1000 → reward_per_token_stored зросте на 100*1000*1e9/1e6 = 1e8
    warp_clock_secs(&mut svm, 100);

    // Зміна rate — `update_reward_per_token` фіксує старе нараховане
    do_update_rate(&mut svm, &pool, &authority, 5_000).unwrap();

    let pool_after = load_pool(&svm, &pool);
    assert_eq!(pool_after.reward_rate, 5_000);
    // Накопичений індекс ≥ 100*1000*1e9/1e6 = 100_000_000 (за старою ставкою)
    assert!(
        pool_after.reward_per_token_stored >= 100_000_000,
        "reward_per_token_stored мав закумулюватися за старою ставкою, отримано: {}",
        pool_after.reward_per_token_stored
    );
}
