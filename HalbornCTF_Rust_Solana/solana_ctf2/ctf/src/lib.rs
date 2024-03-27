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

// this registers the program entrypoint
entrypoint!(process_instruction);

/// this is the program entrypoint
/// this function ALWAYS takes three parameters:
/// the ID of this program, array of accounts and instruction data  
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // process the instruction
    if let Err(error) = processor::Processor::process(program_id, accounts, _instruction_data) {
        // revert the transaction and print the relevant error to validator log if processing fails
        error.print::<error::FarmError>();
        Err(error)
    } else {
        // otherwise return OK
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use {
        super::*, crate::{constant::FARM_FEE, instruction::{ix_pay_create_fee, FarmInstruction}, state::Farm}, assert_matches::*, borsh::BorshSerialize, solana_program::instruction::{AccountMeta, Instruction}, solana_program_test::*, solana_sdk::{clock::Epoch, fee, program_option::COption, signature::{Keypair, Signer}, transaction::Transaction}, spl_token::state::{Account as TokenAccount, AccountState, GenericTokenAccount}, std::convert::TryInto, solana_program::program_pack::Pack
    };

    #[tokio::test]
    async fn test_transaction() {
        // Initialize test environment
        
        // Create accounts needed for the test
        //let farm_id_pubkey = Pubkey::new_unique();
        let authority_pubkey = Pubkey::new_unique();
        let creator_pubkey = Pubkey::new_unique();
        let creator_token_account_pubkey = Pubkey::new_unique();
        let fee_vault_pubkey = Pubkey::new_unique();
        let token_program_id_pubkey = Pubkey::new_unique();
        let farm_program_id_pubkey = Pubkey::new_unique();
        let amount = 5000;
        let program_id = farm_program_id_pubkey;

        let farm_data = Farm {
            enabled: 0,
            nonce: 42,
            token_program_id: token_program_id_pubkey,
            creator: creator_pubkey,
            fee_vault: fee_vault_pubkey,
        };

        let serialized_farm_data = farm_data.try_to_vec().unwrap();

        let authority_key = &authority_pubkey;
        let authority_is_signer = false;
        let authority_is_writable = false;
        let authority_lamports = &mut 1000000000u64;
        let authority_data = &mut [0u8];
        let authority_owner = Pubkey::new_unique();
        let authority_owner_key = &authority_owner;
        let authority_executable = false;
        let authority_epoch = Epoch::default();

        let mut authority_account = AccountInfo::new(
            authority_key, 
            authority_is_signer, 
            authority_is_writable, 
            authority_lamports, 
            authority_data, 
            authority_owner_key, 
            authority_executable, 
            authority_epoch
        );

        let farm_id_key = &Pubkey::create_with_seed(&authority_account.key, "farm", &program_id)
            .expect("Failed to create farm account key");

        //let farm_id_key = &farm_id.pubkey();
        let farm_id_is_signer = false;
        let farm_id_is_writable = true;
        let farm_id_lamports = &mut 1000000000u64;
        let farm_id_data = &mut vec![0u8; serialized_farm_data.len()];
        farm_id_data.copy_from_slice(&serialized_farm_data);
        let farm_id_owner = Pubkey::new_unique();
        let farm_id_owner_key = &farm_id_owner;
        let farm_id_executable = false;
        let farm_id_epoch = Epoch::default();

        let farm_id_account = AccountInfo::new(
            farm_id_key, 
            farm_id_is_signer, 
            farm_id_is_writable, 
            farm_id_lamports, 
            farm_id_data, 
            farm_id_owner_key, 
            farm_id_executable, 
            farm_id_epoch
        );

        // Compute the authority address using the program ID, farm account key, and nonce
        let authority_address = processor::Processor::authority_id(&program_id, farm_id_key, farm_data.nonce)
        .expect("Failed to compute authority address");
        println!("Imprimiendo la authority_Address: {}", authority_address);
        authority_account.key = &authority_address;
    

        let creator_key = &creator_pubkey;
        let creator_is_signer = true;
        let creator_is_writable = false;
        let creator_lamports = &mut 1000000000u64;
        let creator_data = &mut [0u8];
        let creator_owner = Pubkey::new_unique();
        let creator_owner_key = &creator_owner;
        let creator_executable = false;
        let creator_epoch = Epoch::default();

        let creator_account = AccountInfo::new(
            creator_key, 
            creator_is_signer, 
            creator_is_writable, 
            creator_lamports, 
            creator_data, 
            creator_owner_key, 
            creator_executable, 
            creator_epoch
        );

        let creator_token_key = &creator_token_account_pubkey;
        let creator_token_is_signer = false;
        let creator_token_is_writable = false;
        let creator_token_lamports = &mut 1000000000u64;
        let creator_token_data = &mut [0u8];
        let creator_token_owner = Pubkey::new_unique();
        let creator_token_owner_key = &creator_token_owner;
        let creator_token_executable = false;
        let creator_token_epoch = Epoch::default();

        let creator_token_account = AccountInfo::new(
            creator_token_key, 
            creator_token_is_signer, 
            creator_token_is_writable, 
            creator_token_lamports, 
            creator_token_data, 
            creator_token_owner_key, 
            creator_token_executable, 
            creator_token_epoch
        );

        let fee_vault_data = TokenAccount {
            mint: Pubkey::new_unique(),
            owner: *authority_account.key,
            amount: 10000000u64,
            delegate: COption::None,
            state: AccountState::Uninitialized,
            is_native: COption::None,
            delegated_amount: 10000000,
            close_authority: COption::None
        };

        // Serialize the Farm struct instance
        let mut serialized_fee_vault_data_vec : [u8; 165] = [0; 165];
        TokenAccount::pack_into_slice(&fee_vault_data, &mut serialized_fee_vault_data_vec);

        let fee_vault_key = &fee_vault_pubkey;
        let fee_vault_is_signer = false;
        let fee_vault_is_writable = false;
        let fee_vault_lamports = &mut 1000000000u64;
        let fee_vault_data_def = &mut serialized_fee_vault_data_vec;
        let fee_vault_owner = Pubkey::new_unique();
        let fee_vault_owner_key = &fee_vault_owner;
        let fee_vault_executable = false;
        let fee_vault_epoch = Epoch::default();

        let fee_vault_account = AccountInfo::new(
            fee_vault_key, 
            fee_vault_is_signer, 
            fee_vault_is_writable, 
            fee_vault_lamports, 
            fee_vault_data_def, 
            fee_vault_owner_key, 
            fee_vault_executable, 
            fee_vault_epoch
        );

        let token_program_id_key = &token_program_id_pubkey;
        let token_program_id_is_signer = false;
        let token_program_id_is_writable = false;
        let token_program_id_lamports = &mut 1000000000u64;
        let token_program_id_data = &mut [0u8];
        let token_program_id_owner = Pubkey::new_unique();
        let token_program_id_owner_key = &token_program_id_owner;
        let token_program_id_executable = false;
        let token_program_id_epoch = Epoch::default();

        let token_program_id_account = AccountInfo::new(
            token_program_id_key, 
            token_program_id_is_signer, 
            token_program_id_is_writable, 
            token_program_id_lamports, 
            token_program_id_data, 
            token_program_id_owner_key, 
            token_program_id_executable, 
            token_program_id_epoch
        );

        let farm_program_id_key = &farm_program_id_pubkey;
        let farm_program_id_is_signer = false;
        let farm_program_id_is_writable = false;
        let farm_program_id_lamports = &mut 1000000000u64;
        let farm_program_id_data = &mut [0u8];
        let farm_program_id_owner = Pubkey::new_unique();
        let farm_program_id_owner_key = &farm_program_id_owner;
        let farm_program_id_executable = false;
        let farm_program_id_epoch = Epoch::default();

        let farm_program_id_account = AccountInfo::new(
            farm_program_id_key, 
            farm_program_id_is_signer, 
            farm_program_id_is_writable, 
            farm_program_id_lamports, 
            farm_program_id_data, 
            farm_program_id_owner_key, 
            farm_program_id_executable, 
            farm_program_id_epoch
        );

        let mut accounts = [
            farm_id_account,
            authority_account,
            creator_account,
            creator_token_account,
            fee_vault_account,
            token_program_id_account,
            farm_program_id_account
        ];

        // Instruction data
        let instruction_data = FarmInstruction::PayFarmFee(amount);
        let serialized_instruction = instruction_data.try_to_vec().unwrap();

        let result = process_instruction(&program_id, &accounts, &serialized_instruction);
        println!("{:?}", result);

    }

    #[tokio::test]
    async fn test_transaction2() {
        // Initialize test environment
        
        // Create accounts needed for the test
        let farm_id = Keypair::new();
        let authority = Keypair::new();
        let creator = Keypair::new();
        let creator_token_account = Keypair::new();
        let fee_vault = Keypair::new();
        let token_program_id = Keypair::new();
        let farm_program_id = Keypair::new();
        let amount = 5000;

        let mut program_test = ProgramTest::new(
            "ctf-solana-farm", // Specify your program name here
            farm_program_id.pubkey(),
            processor!(process_instruction),
        );

        // Start the test environment
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let instruction = ix_pay_create_fee(
            &farm_id.pubkey(),
            &authority.pubkey(),
            &creator.pubkey(),
            &creator_token_account.pubkey(),
            &fee_vault.pubkey(),
            &token_program_id.pubkey(),
            &farm_program_id.pubkey(),
            amount
        );

        // Initialize the farm account
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &creator], recent_blockhash);
        let result = banks_client.process_transaction(transaction).await;
        println!("PRINT ERROR: {:?}", result);
        assert!(result.is_ok(), "Failed to initialize farm account");
    }

    #[tokio::test]
    async fn test_transaction3() {
        // Initialize test environment
        // Create accounts needed for the test

        let farm_id = Keypair::new();
        let authority = Keypair::new();
        let creator = Keypair::new();

        let mut program_test = ProgramTest::new(
            "ctf-solana-farm", // Specify your program name here
            farm_id.pubkey(),
            processor!(process_instruction),
        );

        // Start the test environment
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let accounts = vec![
            AccountMeta::new(farm_id.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), false),
            AccountMeta::new(creator.pubkey(), true),
        ];
        let nonce = 0; // Set the nonce for the test

        let instruction = Instruction {
            program_id: farm_id.pubkey(),
            accounts,
            data: FarmInstruction::Create { nonce: (nonce) }.try_to_vec().unwrap()
        };

        // Initialize the farm account
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &creator], recent_blockhash);
        let result = banks_client.process_transaction(transaction).await;
        println!("PRINT ERROR: {:?}", result);
        assert!(result.is_ok(), "Failed to initialize farm account");
    }
}
