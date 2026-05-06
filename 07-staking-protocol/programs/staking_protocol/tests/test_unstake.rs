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

fn user_stake_pda(pool: &Pubkey, user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"user", pool.as_ref(), user.as_ref()], program_id)
}

fn unstake_request_pda(pool: &Pubkey, user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"unstake", pool.as_ref(), user.as_ref()], program_id)
}

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

fn setup_with_stake(
    svm: &mut LiteSVM,
    authority: &Keypair,
    user: &Keypair,
    amount: u64,
) -> (Pubkey, Pubkey, Pubkey) {
    let program_id = staking_protocol::id();
    let stake_mint = create_mint(svm, authority, 6);
    let reward_mint = create_mint(svm, authority, 6);
    let (pool, _) = pool_pda(&stake_mint, &program_id);
    let (vault, _) = vault_pda(&pool, &program_id);

    let ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::InitializePool { reward_rate: 1_000 }.data(),
        staking_protocol::accounts::InitializePool {
            authority: authority.pubkey(),
            pool,
            vault,
            stake_mint,
            reward_mint,
            token_program: spl_token::id(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send(svm, ix, &[authority]).unwrap();

    let (user_stake, _) = user_stake_pda(&pool, &user.pubkey(), &program_id);
    let user_token_account =
        create_token_account_with_tokens(svm, authority, &stake_mint, &user.pubkey(), amount);

    let ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::Stake { amount }.data(),
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
    send(svm, ix, &[user]).unwrap();

    (stake_mint, pool, vault)
}

#[test]
fn test_unstake_decreases_total_staked() {
    let program_id = staking_protocol::id();
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/staking_protocol.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let (_stake_mint, pool, _vault) = setup_with_stake(&mut svm, &authority, &user, 1_000_000);
    let (user_stake, _) = user_stake_pda(&pool, &user.pubkey(), &program_id);
    let (unstake_request, _) = unstake_request_pda(&pool, &user.pubkey(), &program_id);

    let ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::Unstake { amount: 400_000 }.data(),
        staking_protocol::accounts::Unstake {
            user: user.pubkey(),
            pool,
            user_stake,
            unstake_request,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send(&mut svm, ix, &[&user]).unwrap();

    // pool.total_staked має зменшитись на 400_000
    let pool_account = svm.get_account(&pool).unwrap();
    let pool_data: staking_protocol::state::StakingPool =
        anchor_lang::AccountDeserialize::try_deserialize(&mut pool_account.data.as_slice())
            .unwrap();
    assert_eq!(pool_data.total_staked, 600_000);

    // UnstakeRequest має існувати з правильними даними
    let req_account = svm.get_account(&unstake_request).unwrap();
    let req_data: staking_protocol::state::UnstakeRequest =
        anchor_lang::AccountDeserialize::try_deserialize(&mut req_account.data.as_slice()).unwrap();
    assert_eq!(req_data.amount, 400_000);
    assert_eq!(req_data.owner, user.pubkey());
}

#[test]
fn test_unstake_more_than_staked_fails() {
    let program_id = staking_protocol::id();
    let authority = Keypair::new();
    let user = Keypair::new();
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/staking_protocol.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let (_stake_mint, pool, _vault) = setup_with_stake(&mut svm, &authority, &user, 1_000_000);
    let (user_stake, _) = user_stake_pda(&pool, &user.pubkey(), &program_id);
    let (unstake_request, _) = unstake_request_pda(&pool, &user.pubkey(), &program_id);

    let ix = Instruction::new_with_bytes(
        program_id,
        &staking_protocol::instruction::Unstake { amount: 2_000_000 }.data(),
        staking_protocol::accounts::Unstake {
            user: user.pubkey(),
            pool,
            user_stake,
            unstake_request,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(
        send(&mut svm, ix, &[&user]).is_err(),
        "unstake більше ніж є мав би зафейлитись"
    );
}
