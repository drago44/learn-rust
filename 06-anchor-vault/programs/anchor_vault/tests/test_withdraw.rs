use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn get_vault_state_pda(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"state", owner.as_ref()], program_id)
}

fn get_vault_pda(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", owner.as_ref()], program_id)
}

fn setup_with_deposit(deposit_amount: u64) -> (LiteSVM, Keypair, Pubkey, Pubkey) {
    let program_id = anchor_vault::id();
    let owner = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();

    let (vault_state, _) = get_vault_state_pda(&owner.pubkey(), &program_id);
    let (vault, _) = get_vault_pda(&owner.pubkey(), &program_id);

    // initialize
    let ix = Instruction::new_with_bytes(
        program_id,
        &anchor_vault::instruction::Initialize {}.data(),
        anchor_vault::accounts::Initialize {
            owner: owner.pubkey(),
            vault_state,
            vault,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&owner.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&owner]).unwrap();
    svm.send_transaction(tx).unwrap();

    // deposit
    let ix = Instruction::new_with_bytes(
        program_id,
        &anchor_vault::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
        anchor_vault::accounts::Deposit {
            owner: owner.pubkey(),
            vault_state,
            vault,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&owner.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&owner]).unwrap();
    svm.send_transaction(tx).unwrap();

    (svm, owner, vault_state, vault)
}

#[test]
fn test_withdraw() {
    let deposit_amount = 2_000_000_000u64; // 2 SOL
    let withdraw_amount = 1_000_000_000u64; // 1 SOL
    let (mut svm, owner, vault_state, vault) = setup_with_deposit(deposit_amount);
    let program_id = anchor_vault::id();

    let owner_balance_before = svm.get_balance(&owner.pubkey()).unwrap_or(0);
    let vault_balance_before = svm.get_balance(&vault).unwrap_or(0);

    let ix = Instruction::new_with_bytes(
        program_id,
        &anchor_vault::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
        anchor_vault::accounts::Withdraw {
            owner: owner.pubkey(),
            vault_state,
            vault,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&owner.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&owner]).unwrap();
    let res = svm.send_transaction(tx);

    assert!(res.is_ok(), "withdraw failed: {:?}", res.err());

    let vault_balance_after = svm.get_balance(&vault).unwrap_or(0);
    let owner_balance_after = svm.get_balance(&owner.pubkey()).unwrap_or(0);

    assert_eq!(vault_balance_before - vault_balance_after, withdraw_amount);
    // власник отримав SOL (мінус комісія за транзакцію)
    assert!(owner_balance_after > owner_balance_before);
}

#[test]
fn test_withdraw_full() {
    let deposit_amount = 1_000_000_000u64;
    let (mut svm, owner, vault_state, vault) = setup_with_deposit(deposit_amount);
    let program_id = anchor_vault::id();

    let ix = Instruction::new_with_bytes(
        program_id,
        &anchor_vault::instruction::Withdraw {
            amount: deposit_amount,
        }
        .data(),
        anchor_vault::accounts::Withdraw {
            owner: owner.pubkey(),
            vault_state,
            vault,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&owner.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&owner]).unwrap();
    let res = svm.send_transaction(tx);

    assert!(res.is_ok(), "full withdraw failed: {:?}", res.err());
    assert_eq!(svm.get_balance(&vault).unwrap_or(0), 0);
}
