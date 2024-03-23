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
    use std::str::FromStr;

    use crate::constant::{FARM_FEE, FEE_OWNER, USDC_MINT_ADDRESS};
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


    // Include the SyscallStubs struct and its implementation
    struct SyscallStubs {}

    impl solana_sdk::program_stubs::SyscallStubs for SyscallStubs {
        fn sol_log(&self, message: &str) {
            println!("{}", message);
        }

        fn sol_invoke_signed(
            &self,
            _instruction: &Instruction,
            _account_infos: &[AccountInfo],
            _signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            Self::sol_log(&self, ":))");
            Ok(())
        }
        
    }

    // Define your test functions
    #[tokio::test]
    async fn test_initialize_farm() {

        let program_id = Pubkey::new_unique();
        // Set up the test environment with SyscallStubs
        let program_test = ProgramTest::new(
            "ctf-solana-farm", // Name of your program
            program_id,          // Program ID
            processor!(process_instruction), // Entrypoint function
        );

        let mut banks_client = program_test.start_with_context();        
        // Write your test logic using banks_client
    }

    #[tokio::test]
    async fn test_initialize_farm2() {
        solana_sdk::program_stubs::set_syscall_stubs(Box::new(SyscallStubs {}));
        
        let program_id = Pubkey::new_unique();
        println!("Test");
        // Authority info creation
        let authority_info_key = &Pubkey::new_unique();
        let authority_info_lamports = &mut 10000;
        let authority_info_data = &mut [0u8];
        let authority_info_owner = &Pubkey::new_unique();
        let mut authority_info = AccountInfo::new(
            &authority_info_key,
            false,
            false,
            authority_info_lamports,
            authority_info_data,
            authority_info_owner,
            false,
            Epoch::default(),
        );

        // Creator info creation
        let creator_info_key = &Pubkey::new_unique();
        let creator_info_lamports = &mut 10000;
        let creator_info_data = &mut [0u8];
        let creator_info_owner = &Pubkey::new_unique();
        let mut creator_info = AccountInfo::new(
            creator_info_key,
            false,
            false,
            creator_info_lamports,
            creator_info_data,
            creator_info_owner,
            false,
            Epoch::default(),
        );

        // user_transfer_authority info creation
        let user_transfer_authority_info_key = &Pubkey::new_unique();
        let user_transfer_authority_info_lamports = &mut 10000;
        let user_transfer_authority_info_data = &mut [0u8];
        let user_transfer_authority_info_owner = &Pubkey::new_unique();
        let mut user_transfer_authority_info = AccountInfo::new(
            user_transfer_authority_info_key,
            false,
            false,
            user_transfer_authority_info_lamports,
            user_transfer_authority_info_data,
            user_transfer_authority_info_owner,
            false,
            Epoch::default(),
        );

        // user_usdc_token_account info creation
        let user_usdc_token_account_info_key = &Pubkey::from_str(USDC_MINT_ADDRESS).unwrap();
        let user_usdc_token_account_info_lamports = &mut 10000;
        let user_usdc_token_account_info_data = &mut [0u8];
        let user_usdc_token_account_info_owner = &Pubkey::new_unique();
        let mut user_usdc_token_account_info = AccountInfo::new(
            user_usdc_token_account_info_key,
            false,
            false,
            user_usdc_token_account_info_lamports,
            user_usdc_token_account_info_data,
            user_usdc_token_account_info_owner,
            false,
            Epoch::default(),
        );

        // fee_owner info creation
        let fee_owner_info_key = &Pubkey::from_str(FEE_OWNER).unwrap();
        let fee_owner_info_lamports = &mut 10000;
        let fee_owner_info_data = &mut [0u8];
        let fee_owner_info_owner = &Pubkey::new_unique();
        let mut fee_owner_info = AccountInfo::new(
            fee_owner_info_key,
            false,
            false,
            fee_owner_info_lamports,
            fee_owner_info_data,
            fee_owner_info_owner,
            false,
            Epoch::default(),
        );

        // token_program info creation
        let token_program_info_key = &TOKEN_PROGRAM_ID();
        let token_program_info_lamports = &mut 10000;
        let token_program_info_data = &mut [0u8];
        let token_program_info_owner = &Pubkey::new_unique();
        let mut token_program_info = AccountInfo::new(
            token_program_info_key,
            false,
            false,
            token_program_info_lamports,
            token_program_info_data,
            &token_program_info_owner,
            false,
            Epoch::default(),
        );

        // Create an instance of the Farm struct
        let farm_instance = Farm {
            is_allowed: 0,
            nonce: 42,
            pool_lp_token_account: Pubkey::new_unique(),
            pool_reward_token_account: Pubkey::new_unique(),
            pool_mint_address: Pubkey::new_unique(),
            reward_mint_address: Pubkey::new_unique(),
            token_program_id: *token_program_info.key,
            owner: *creator_info_key,
            fee_owner: *fee_owner_info.key,
            reward_per_share_net: 100,
            last_timestamp: 1647024000,
            reward_per_timestamp: 10,
            start_timestamp: 1647024000,
            end_timestamp: 1647110400,
        };

        // Serialize the Farm struct instance
        let serialized_data: Vec<u8> = farm_instance.try_to_vec().unwrap();

        // Farm account creation
        //let farm_account_info_key_aux = &Pubkey::new_unique();
        let farm_account_info_key = &Pubkey::create_with_seed(&authority_info.key, "farm", &program_id)
            .expect("Failed to create farm account key");
        println!("Farm account info key created: {}", farm_account_info_key);
        let farm_account_info_lamports = &mut 10000;

        // Use the serialized_data to initialize farm_account_info_data
        let farm_account_info_data = &mut vec![0u8; serialized_data.len()];
        farm_account_info_data.copy_from_slice(&serialized_data);
        let farm_account_info_owner = &Pubkey::new_unique();
        let mut farm_account_info = AccountInfo::new(
            farm_account_info_key,
            false,
            true,
            farm_account_info_lamports,
            farm_account_info_data,
            farm_account_info_owner,
            true,
            Epoch::default(),
        );

        // Compute the authority address using the program ID, farm account key, and nonce
        let authority_address = processor::Processor::authority_id(&program_id, farm_account_info_key, farm_instance.nonce)
        .expect("Failed to compute authority address");
        println!("Imprimiendo la authority_Address: {}", authority_address);
        authority_info.key = &authority_address;
        
        let mut accounts = [
            farm_account_info.clone(),
            authority_info.clone(),
            creator_info.clone(),
            user_transfer_authority_info.clone(),
            user_usdc_token_account_info.clone(),
            fee_owner_info.clone(),
            token_program_info.clone(),
        ];

        // Example instruction data
        let instruction_data = FarmInstruction::PayFarmFee(FARM_FEE);

        // Serialize the instruction data
        let serialized_instruction = instruction_data.try_to_vec().unwrap();

        if let Err(error) = processor::Processor::process(&program_id, &accounts, &serialized_instruction) {
            error.print::<error::FarmError>();
        } else {
            println!("No error");
            println!("Farm is allowed: {}", farm_instance.is_allowed);
            for account in accounts {
                println!("Lamports: {}", account.lamports());
            }
            println!("Serialized instruction: {:?}", serialized_instruction);
        }

    }
}
