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


/*#[cfg(test)]
mod test {
    use {
        super::*, crate::{constant::FARM_FEE, error::FarmError, instruction::{ix_create_farm, ix_pay_create_fee, FarmInstruction}, state::Farm}, assert_matches::*, borsh::BorshSerialize, solana_program::{instruction::{AccountMeta, Instruction}, program_pack::Pack}, solana_program_test::{tokio::time::sleep, *}, solana_sdk::{account::Account, clock::Epoch, config::program, fee, msg, program_option::COption, rent::Rent, signature::{Keypair, Signer}, system_program, transaction::Transaction}, spl_token::state::{Account as TokenAccount, AccountState, GenericTokenAccount}, std::{convert::TryInto, mem::size_of, str::FromStr, time::Duration}
    };

    #[tokio::test]
    async fn test_transaction() {
        
        // Create accounts needed for the test
        let program_id = Pubkey::new_unique();
        let creator = Keypair::new();
        let creator_token = Keypair::new();
        let fee_vault = Keypair::new();
        let token_program_id = Keypair::new();
        let farm_program_id = Keypair::new();
        let amount = 5000;

        // Create the ProgramTest object
        let mut program_test = ProgramTest::new(
            "ctf-solana-farm", // Specify your program name here
            program_id,
            processor!(process_instruction),
        );

        // Create and initialize the accounts

        // Farm id account initialization
        let farm_id_key = Pubkey::new_unique();
        let farm_id_account_data_size = size_of::<Farm>();
        let farm_id_lamports = 10000;
        let farm_id_owner = Pubkey::new_unique();
        let mut farm_id_account = Account::new(farm_id_lamports, farm_id_account_data_size, &farm_id_owner);
        let farm_data = Farm {
            enabled: 0,
            nonce: 42,
            token_program_id: token_program_id.pubkey(),
            creator: creator.pubkey(),
            fee_vault: fee_vault.pubkey(),
        };
        farm_id_account.data[..].copy_from_slice(&farm_data.try_to_vec().unwrap());
        
        // Authority account initialization
        let authority_key = match Pubkey::create_program_address(&[&farm_id_key.to_bytes()[..32], &[farm_data.nonce]], &program_id) {
            Ok(address) => address,
            Err(err) => panic!("Failed to generate authority address: {:?}", err),
        };
        let authority_account_data_size = 0;
        let authority_account_lamports = 10000;
        let authority_owner = Pubkey::new_unique();
        let mut authority_account = Account::new(authority_account_lamports, authority_account_data_size, &authority_owner);

        // Creator account initialization
        let creator_account_data_size = 0;
        let creator_account_lamports = 10000;
        let creator_account_owner = Pubkey::new_unique();
        let mut creator_account = Account::new(creator_account_lamports, creator_account_data_size, &creator_account_owner);

        // Creator token account initialization
        let creator_token_account_data_size = 0;
        let creator_token_account_lamports = 10000;
        let creator_token_account_owner = Pubkey::new_unique();
        let mut creator_token_account = Account::new(creator_token_account_lamports, creator_token_account_data_size, &creator_token_account_owner);

        // Fee vault account initialization
        let fee_vault_data = TokenAccount {
            mint: Pubkey::new_unique(),
            owner: authority_key,
            amount: 10000000u64,
            delegate: COption::Some(authority_key),
            state: AccountState::Uninitialized,
            is_native: COption::Some(1),
            delegated_amount: 10000000,
            close_authority: COption::Some(authority_key)
        };

        // Serialize the Farm struct instance
        let mut serialized_fee_vault_data_vec : [u8; 165] = [0; 165];
        TokenAccount::pack_into_slice(&fee_vault_data, &mut serialized_fee_vault_data_vec);
        let fee_vault_account_data_size = 165;
        let fee_vault_account_lamports = 10000;
        let fee_vault_account_owner = Pubkey::new_unique();
        let mut fee_vault_account = Account::new(fee_vault_account_lamports, fee_vault_account_data_size, &fee_vault_account_owner);
        fee_vault_account.data[..].copy_from_slice(&serialized_fee_vault_data_vec);

        // Farm program id account initialization
        let farm_program_id_account_data_size = 0;
        let farm_program_id_account_lamports = 10000;
        let mut farm_program_id_account = Account::new(farm_program_id_account_lamports, farm_program_id_account_data_size, &program_id);
        let farm_program_id_key = program_id;

        // Token program id account initialization
        let token_program_id_account_data_size = 0;
        let token_program_id_account_lamports = 10000;
        let mut token_program_id_account = Account::new(token_program_id_account_lamports, token_program_id_account_data_size, &program_id);
        token_program_id_account.executable = true;
        //let token_program_id_key = Pubkey::from_str("CzsHNvU2hk2tnoSXLBoJcLJeoFVhJqA1QCDvwF77pqf1").unwrap();
        //Pubkey::from_str("CzsHNvU2hk2tnoSXLBoJcLJeoFVhJqA1QCDvwF77pqf1").unwrap();
        let token_program_id_key = spl_token::ID;
        
        // Add the initialized accounts to the program test environment
        program_test.add_account(farm_id_key, farm_id_account);
        program_test.add_account(authority_key, authority_account);
        program_test.add_account(creator.pubkey(), creator_account);
        program_test.add_account(creator_token.pubkey(), creator_token_account);
        program_test.add_account(fee_vault.pubkey(), fee_vault_account);
        program_test.add_account(farm_program_id_key, farm_program_id_account);
        program_test.add_account(token_program_id_key, token_program_id_account);

        // Start the test environment
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        
        // Create the instruction
        let instruction = ix_pay_create_fee(
            &farm_id_key,
            &authority_key,
            &creator.pubkey(),
            &creator_token.pubkey(),
            &fee_vault.pubkey(),
            &token_program_id_key,
            &farm_program_id_key,
            amount
        );

        // Create the transaction
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );

        // Sign the transaction
        transaction.sign(&[&payer, &creator], recent_blockhash);
        let result = banks_client.process_transaction(transaction).await;
        msg!("PRINT ERROR: {:?}", result);

        // Check that the result of the transaction is ok
        assert!(result.is_ok(), "Failed to initialize farm account");
    }

    /*#[tokio::test]
    async fn test_transaction2() {
        
        // Create account keypairs needed for the test
        let program_id = Pubkey::new_unique();
        let farm_id = Keypair::new();
        let authority = Keypair::new();
        let creator = Keypair::new();
        let creator_token = Keypair::new();
        let fee_vault = Keypair::new();
        let token_program_id = Keypair::new();
        let farm_program_id = Keypair::new();
        let amount = 5000;

        // Create the ProgramTest object
        let mut program_test = ProgramTest::new(
            "ctf-solana-farm",
            program_id,
            processor!(process_instruction),
        );

        msg!("THE PROGRAM ID IS: {}", program_id);

        // Create and initialize the accounts

        // Farm id account initialization
        let farm_id_account_data_size = size_of::<Farm>();
        let farm_id_lamports = 10000;
        let mut farm_id_account = Account::new(farm_id_lamports, farm_id_account_data_size, &farm_program_id.pubkey());
        let farm_data = Farm {
            enabled: 0,
            nonce: 42,
            token_program_id: token_program_id.pubkey(),
            creator: creator.pubkey(),
            fee_vault: fee_vault.pubkey(),
        };
        farm_id_account.data[..].copy_from_slice(&farm_data.try_to_vec().unwrap());
        // Generate farm ID key using the same seeds as in your program logic
        let farm_id_key = Pubkey::new_unique();
        let seeds = [&farm_id_key.to_bytes()[..32], &[farm_data.nonce]];
        let (farm_id_key, _) = Pubkey::find_program_address(&seeds, &program_id);

        // Authority account initialization
        let authority_address = processor::Processor::authority_id(&farm_program_id.pubkey(), &farm_id_key, farm_data.nonce)
        .expect("Failed to compute authority address");

        let authority_account_data_size = 0;
        let authority_account_lamports = 10000;
        let mut authority_account = Account::new(authority_account_lamports, authority_account_data_size, &farm_program_id.pubkey());

        // Creator account initialization
        let creator_account_data_size = 0;
        let creator_account_lamports = 10000;
        let mut creator_account = Account::new(creator_account_lamports, creator_account_data_size, &farm_program_id.pubkey());

        // Farm program id account initialization
        let farm_program_id_account_data_size = 0;
        let farm_program_id_account_lamports = 10000;
        let mut farm_program_id_account = Account::new(farm_program_id_account_lamports, farm_program_id_account_data_size, &farm_program_id.pubkey());

        // Add the initialized accounts to the program test environment
        program_test.add_account(farm_id_key, farm_id_account);
        program_test.add_account(authority_address, authority_account);
        program_test.add_account(creator.pubkey(), creator_account);
        program_test.add_account(farm_program_id.pubkey(), farm_program_id_account);

        // Start the test environment
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let nonce = 42;
        // Create the instruction
        let instruction = ix_create_farm(
            &farm_id_key,
            &authority_address,
            &creator.pubkey(),
            &farm_program_id.pubkey(),
            nonce
        );

        // Create the transaction
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );

        // Sign the transaction
        transaction.sign(&[&payer, &creator], recent_blockhash);
        let result = banks_client.process_transaction(transaction).await;
        println!("PRINT ERROR: {:?}", result);

        // Check that the result of the transaction is ok
        assert!(result.is_ok(), "Failed to initialize farm account");
    }*/
}*/


