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
    use std::convert::TryInto;
    use std::mem;

    use crate::instruction::FarmInstruction;
    use crate::state::Farm;

    use super::*;
    use borsh::BorshSerialize;
    use solana_program_test::*;
    use solana_sdk::account::Account;
    use solana_sdk::borsh0_10::try_from_slice_unchecked;
    use solana_sdk::instruction::AccountMeta;
    use solana_sdk::signature::{Keypair, Signer};
    use solana_sdk::system_instruction;

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

    #[tokio::test]
    async fn test_process_instruction() {
        // Set up the test environment
        let program_id = Pubkey::new_unique();
        println!("Program id: {}", program_id.to_string());
        let mut program_test = ProgramTest::new(
            "ctf-solana-farm", // Change this to your program name
            program_id,
            processor!(process_instruction),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Create test accounts
        let farm_account = Keypair::new();
        //println!("Farm account: {}", farm_account.to_base58_string());
        let authority = Keypair::new();
        //println!("Authority: {}", authority.to_base58_string());
        let creator = Keypair::new();
        let lp_token_account = Keypair::new();
        let user_transfer_authority = Keypair::new();
        let user_usdc_token_account = Keypair::new();
        let fee_owner = Keypair::new();
        let token_program = Keypair::new();
        let farm_program_id_keypair = Keypair::new();
        let farm_program_id = farm_program_id_keypair.pubkey();
        println!("Farm Program ID: {}", farm_program_id_keypair.pubkey().to_string());

        // Initialize accounts
        let accounts = vec![
            &farm_account, &authority, &creator, &lp_token_account, &user_transfer_authority,
            &user_usdc_token_account, &fee_owner, &token_program, &farm_program_id_keypair,
        ];

        for account in accounts {
            create_account(
                &mut banks_client,
                &payer,
                &recent_blockhash,
                account,
                10000000, // Provide a sufficient balance for initialization
                &program_id,
            )
            .await;
        }

        // Create and initialize the farm
        let create_instruction_data = FarmInstruction::Create {
            nonce: 0,
            start_timestamp: 1647024000,
            end_timestamp: 1647110400,
        };

        let create_instruction = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(farm_account.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(lp_token_account.pubkey(), false),
                AccountMeta::new(user_usdc_token_account.pubkey(), false),
                AccountMeta::new_readonly(Pubkey::new_unique(), false), // Placeholder for pool mint
                AccountMeta::new_readonly(Pubkey::new_unique(), false), // Placeholder for reward mint
                AccountMeta::new_readonly(Pubkey::new_unique(), false), // Placeholder for amm id
                AccountMeta::new_readonly(token_program.pubkey(), false),
                AccountMeta::new_readonly(farm_program_id_keypair.pubkey(), false),
            ],
            data: create_instruction_data.try_to_vec().unwrap(),
        };

        // Sign and send the create transaction
        let mut create_transaction =
            Transaction::new_with_payer(&[create_instruction], Some(&payer.pubkey()));
            println!("Llega aquí");
            create_transaction.sign(
                &[&payer, &farm_account, &creator, &farm_program_id_keypair],
                recent_blockhash,
            );
            println!("Llega aquí 2");
            banks_client
            .process_transaction(create_transaction)
            .await
            .unwrap();

        // Verify the state after farm initialization
        let farm_data = banks_client
            .get_account(farm_account.pubkey())
            .await
            .expect("Failed to get farm account data")
            .unwrap();
        let farm_state: Farm = try_from_slice_unchecked(&farm_data.data)
            .expect("Failed to deserialize farm account data");

        // Now, let's craft an instruction to pay the farm fee
        let pay_fee_instruction_data = FarmInstruction::PayFarmFee(5000);

        let pay_fee_instruction = Instruction {
            program_id: farm_program_id,
            accounts: vec![
                AccountMeta::new(farm_account.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(user_transfer_authority.pubkey(), false),
                AccountMeta::new(user_usdc_token_account.pubkey(), false),
                AccountMeta::new(fee_owner.pubkey(), false),
                AccountMeta::new_readonly(token_program.pubkey(), false),
                AccountMeta::new_readonly(farm_program_id, false),
            ],
            data: pay_fee_instruction_data.try_to_vec().unwrap(),
        };

        // Sign and send the pay fee transaction
        let mut pay_fee_transaction =
            Transaction::new_with_payer(&[pay_fee_instruction], Some(&payer.pubkey()));
        pay_fee_transaction.sign(
            &[&payer, &farm_account, &authority],
            recent_blockhash,
        );
        banks_client
            .process_transaction(pay_fee_transaction)
            .await
            .unwrap();

        // Verify the state after paying the farm fee
        let updated_farm_data = banks_client
            .get_account(farm_account.pubkey())
            .await
            .expect("Failed to get updated farm account data")
            .unwrap();
        let updated_farm_state: Farm = try_from_slice_unchecked(&updated_farm_data.data)
            .expect("Failed to deserialize updated farm account data");

        // Add assertions based on the expected state changes
        assert_eq!(updated_farm_state.is_allowed, 1);
        assert_eq!(updated_farm_state.nonce, 0);
        // Add more assertions based on the actual logic of your program
    }

    // The create_account function remains unchanged
    async fn create_account(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
        account: &Keypair,
        lamports: u64,
        program_id: &Pubkey,
    ) {
        let transaction = Transaction::new_signed_with_payer(
            &[system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                lamports,
                mem::size_of::<Account>().try_into().unwrap(),
                &program_id,
            )],
            Some(&payer.pubkey()),
            &[payer, account],
            *recent_blockhash,
        );

        banks_client.process_transaction(transaction).await.unwrap();
    }
}