use {
    assert_matches::*, borsh::BorshSerialize, ctf_solana_farm::{constant::FARM_FEE, error::FarmError, instruction::{ix_create_farm, ix_pay_create_fee, FarmInstruction}, state::Farm}, solana_program::{instruction::{AccountMeta, Instruction}, 
        program_pack::Pack}, solana_program_test::{tokio::time::sleep, *}, solana_sdk::{account::{Account, AccountSharedData}, clock::Epoch, config::program, fee, msg, native_token::LAMPORTS_PER_SOL, program_option::COption, pubkey::Pubkey, rent::Rent, signature::{Keypair, Signer}, system_program, transaction::Transaction}, solana_validator::test_validator::TestValidatorGenesis, spl_token::state::{Account as TokenAccount, 
        AccountState, 
        GenericTokenAccount}, std::{convert::TryInto, 
        mem::size_of, 
        str::FromStr, 
        time::Duration}
};

use solana_client::rpc_client::RpcClient;
use solana_sdk::account::ReadableAccount;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::system_instruction;

fn main() {

    println!("Here it is the Rust Client!");
    let keypair_payer = read_keypair_file("./keypair_payer").unwrap();
    println!("🔑 The pubkey is: {}", keypair_payer.pubkey());

    let rpc = RpcClient::new("http://127.0.0.1:8899".to_string());
    //let rpc = RpcClient::new("https://api.devnet.solana.com".to_string());
    let recent_blockhash = rpc.get_latest_blockhash().unwrap();
    println!("The latest blockhash is: {}", recent_blockhash);

    let lamports_airdrop = 1 * LAMPORTS_PER_SOL;

    let result = rpc.request_airdrop(&keypair_payer.pubkey(), lamports_airdrop);

    println!("Signature of payer airdrop: {:?}", result);
    println!("Payer balance: {:?}", rpc.get_balance(&keypair_payer.pubkey()));

    let program_id = Pubkey::from_str(&"x5i6Ec4fuLkEBB9cr7PRRndzLk4uPFkskDzWnPeyXHi".to_string()).unwrap();

    println!("The program ID of the CTF is: {}", program_id);

    let program_from_rpc = rpc.get_account(&program_id).unwrap();

    println!("{:?}", program_from_rpc.lamports());

    let creator = Keypair::new();
    let creator_token = Keypair::new();
    let fee_vault = Keypair::new();
    let token_program_id = Keypair::new();
    let farm_program_id = Keypair::new();
    let amount = 5000;

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
    let authority_account_lamports = 1 * LAMPORTS_PER_SOL;
    let authority_owner = Pubkey::new_unique();
    let mut authority_account = Account::new(authority_account_lamports, authority_account_data_size, &authority_owner);

    // Creator account initialization
    let creator_account_data_size = 0;
    let creator_account_lamports = 1 * LAMPORTS_PER_SOL;
    let creator_account_owner = Pubkey::new_unique();
    let mut creator_account = Account::new(creator_account_lamports, creator_account_data_size, &creator_account_owner);

    // Creator token account initialization
    let creator_token_account_data_size = 0;
    let creator_token_account_lamports = 1 * LAMPORTS_PER_SOL;
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
    let fee_vault_account_lamports = 1 * LAMPORTS_PER_SOL;
    let fee_vault_account_owner = Pubkey::new_unique();
    let mut fee_vault_account = Account::new(fee_vault_account_lamports, fee_vault_account_data_size, &fee_vault_account_owner);
    fee_vault_account.data[..].copy_from_slice(&serialized_fee_vault_data_vec);

    // Farm program id account initialization
    let farm_program_id_account_data_size = 0;
    let farm_program_id_account_lamports = 1 * LAMPORTS_PER_SOL;
    let mut farm_program_id_account = Account::new(farm_program_id_account_lamports, farm_program_id_account_data_size, &program_id);
    let farm_program_id_key = program_id;

    // Token program id account initialization
    let token_program_id_account_data_size = 0;
    let token_program_id_account_lamports = 1 * LAMPORTS_PER_SOL;
    let mut token_program_id_account = Account::new(token_program_id_account_lamports, token_program_id_account_data_size, &program_id);
    token_program_id_account.executable = true;
    let token_program_id_key = spl_token::ID;
    println!("{}", token_program_id_key);

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

    println!("{:?}", instruction);

    // Create the transaction
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&keypair_payer.pubkey()),
    );

    // Sign the transaction
    transaction.sign(&[&keypair_payer, &creator], recent_blockhash);

    // Check that the result of the transaction is ok
    assert_matches!(rpc.send_and_confirm_transaction(&transaction), Ok(_));
}