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

#[test]
fn test_initialize() {
    let program_id = anchor_vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let (vault_state, _) = get_vault_state_pda(&payer.pubkey(), &program_id);
    let (vault, _) = get_vault_pda(&payer.pubkey(), &program_id);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_vault::instruction::Initialize {}.data(),
        anchor_vault::accounts::Initialize {
            owner: payer.pubkey(),
            vault_state,
            vault,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());
}
