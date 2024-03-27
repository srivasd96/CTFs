use solana_program::{
    account_info::{ AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::PrintProgramError,
    pubkey::Pubkey,
};

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod constant;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::Processor::process(program_id, accounts, _instruction_data) {
        error.print::<error::FarmError>();
        Err(error)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::convert::TryInto;
    use std::mem;
    use std::str::FromStr;

    use crate::constant::{FARM_FEE, FEE_OWNER, USDC_MINT_ADDRESS};
    use crate::instruction::FarmInstruction;
    use crate::state::Farm;

    use assert_matches::assert_matches;

    use super::*;
    use borsh::BorshSerialize;
    use solana_program_test::*;
    use solana_sdk::account::Account;
    use solana_sdk::borsh0_10::try_from_slice_unchecked;
    use solana_sdk::instruction::AccountMeta;
    use solana_sdk::signature::{Keypair, Signer};
    use solana_sdk::system_instruction;
    
    use spl_token::id as TOKEN_PROGRAM_ID;

    // Import the necessary types from the solana_program module
    use solana_program::instruction::{Instruction};
    use solana_program::program_pack::Pack;
    use solana_program::system_program;
    use solana_program::sysvar::rent;
    use solana_program::sysvar::Sysvar;
    use solana_program::hash::Hash;
    use solana_program::clock::Epoch;
    use solana_program::account_info::{next_account_info, AccountInfo};
    use solana_program::msg;
    use solana_sdk::transaction::Transaction;

    use crate::instruction::ix_pay_create_fee;

    // Define your test functions
    #[tokio::test]
    async fn test_initialize_farm() {

        let farm_program_id = Keypair::new();

        let program_id = farm_program_id.pubkey();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "ctf-solana-farm",
            program_id,
            processor!(process_instruction),
        )
        .start()
        .await;

        let farm_id = Keypair::new();
        let authority = Keypair::new();
        let creator = Keypair::new();
        let creator_token_account = Keypair::new();
        let fee_vault = Keypair::new();
        let token_program_id = Keypair::new();

        let accounts = vec![
            AccountMeta::new(farm_id.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), false),
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(creator_token_account.pubkey(), false),
            AccountMeta::new(fee_vault.pubkey(), false),
            AccountMeta::new_readonly(token_program_id.pubkey(), false),
        ];

        let instruction = Instruction {
            program_id: farm_program_id.pubkey(),
            accounts,
            data: FarmInstruction::Create { nonce: 12, start_timestamp: 1711446463, end_timestamp: 1711792063 }.try_to_vec().unwrap(),
        };
        
        println!("{:?}", instruction);

        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );

        // Include all three signers: payer, authority, and creator
        transaction.sign(&[&payer, &creator, &authority], recent_blockhash);

        assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
    }

    #[tokio::test]
    async fn test_initialize_farm2() {

        let farm_id = Keypair::new();
        let authority = Keypair::new();
        let creator = Keypair::new();
        let creator_token_account = Keypair::new();
        let fee_vault = Keypair::new();
        let token_program_id = Keypair::new();
        let farm_program_id = Keypair::new();
        let amount = 5000;

        let program_id = farm_program_id.pubkey();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "ctf-solana-farm",
            program_id,
            processor!(process_instruction),
        )
        .start()
        .await;

        let instruction = ix_pay_create_fee(
            &farm_id.pubkey(), 
            &authority.pubkey(), 
            &creator.pubkey(), 
            &creator_token_account.pubkey(), 
            &fee_vault.pubkey(), 
            &token_program_id.pubkey(), 
            &farm_program_id.pubkey(), 
            amount);

        println!("{:?}", instruction);

        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );

        // Include all three signers: payer, authority, and creator
        transaction.sign(&[&payer, &creator, &authority], recent_blockhash);

        assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
    }

}
