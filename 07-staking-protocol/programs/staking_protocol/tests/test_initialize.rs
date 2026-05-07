use anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spl_token::state::Mint;

fn pool_pda(stake_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool", stake_mint.as_ref()], program_id)
}

fn vault_pda(pool: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", pool.as_ref()], program_id)
}

fn reward_vault_pda(pool: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"reward_vault", pool.as_ref()], program_id)
}

fn create_mint(svm: &mut LiteSVM, payer: &Keypair, decimals: u8) -> Pubkey {
    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();

    let rent = svm.minimum_balance_for_rent_exemption(Mint::LEN);

    let create_account_ix = solana_system_interface::instruction::create_account(
        &payer.pubkey(),
        &mint_pubkey,
        rent,
        Mint::LEN as u64,
        &spl_token::id(),
    );

    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &payer.pubkey(),
        None,
        decimals,
    )
    .unwrap();

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[create_account_ix, init_mint_ix],
        Some(&payer.pubkey()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer, &mint]).unwrap();
    svm.send_transaction(tx).unwrap();

    mint_pubkey
}

#[test]
fn test_initialize_pool_ok() {
    let program_id = staking_protocol::id();
    let authority = Keypair::new();
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/staking_protocol.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    let stake_mint = create_mint(&mut svm, &authority, 6);
    let reward_mint = create_mint(&mut svm, &authority, 6);
    let (pool, _) = pool_pda(&stake_mint, &program_id);
    let (vault, _) = vault_pda(&pool, &program_id);
    let (reward_vault, _) = reward_vault_pda(&pool, &program_id);

    let ix = Instruction::new_with_bytes(
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

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&authority.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[authority]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "initialize_pool failed: {:?}", res.err());
}

#[test]
fn test_initialize_pool_duplicate_fails() {
    let program_id = staking_protocol::id();
    let authority = Keypair::new();
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/staking_protocol.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    let stake_mint = create_mint(&mut svm, &authority, 6);
    let reward_mint = create_mint(&mut svm, &authority, 6);
    let (pool, _) = pool_pda(&stake_mint, &program_id);
    let (vault, _) = vault_pda(&pool, &program_id);
    let (reward_vault, _) = reward_vault_pda(&pool, &program_id);

    let make_ix = || {
        Instruction::new_with_bytes(
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
        )
    };

    let send = |svm: &mut LiteSVM, ix: Instruction, signer: &Keypair| {
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&signer.pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
        svm.send_transaction(tx)
    };

    assert!(send(&mut svm, make_ix(), &authority).is_ok());
    assert!(
        send(&mut svm, make_ix(), &authority).is_err(),
        "повторна ініціалізація мала б зафейлитись"
    );
}
