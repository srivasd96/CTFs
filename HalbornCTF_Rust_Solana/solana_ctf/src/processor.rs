use {
    crate::{
        error::FarmError,
        instruction::{
            FarmInstruction
        },
        state::{
            Farm,
        },
        constant::{
            FARM_FEE,
        },
    },
    borsh::{BorshDeserialize, BorshSerialize},
    num_traits::FromPrimitive,
    solana_program::{
        account_info::{
            next_account_info,
            AccountInfo,
        },
        borsh0_10::try_from_slice_unchecked,
        decode_error::DecodeError,
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::PrintProgramError,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

pub struct Processor {}
impl Processor {  
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = FarmInstruction::try_from_slice(input)?;
        match instruction {
            FarmInstruction::PayFarmFee(amount) => {
                Self::process_pay_farm_fee(program_id, accounts, amount)
            },

            _ => Err(FarmError::NotAllowed.into())
        }
    } 

    pub fn process_pay_farm_fee(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        msg!("Entra");
        let account_info_iter = &mut accounts.iter();
        let farm_id_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let creator_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let user_usdc_token_account_info = next_account_info(account_info_iter)?;
        let fee_owner_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        msg!("Entra 1b");
        let mut farm_data = try_from_slice_unchecked::<Farm>(&farm_id_info.data.borrow())?;
        msg!("Entra 2");
        if farm_data.is_allowed == 1 {
            return Err(FarmError::AlreadyInUse.into());
        }

        if *creator_info.key != farm_data.owner {
            return Err(FarmError::WrongManager.into());
        }
        
        println!("Program id: {}", program_id);
        println!("Farm id: {}", farm_id_info.key);
        println!("Farm data nonce: {}", farm_data.nonce);

        println!("Primera parte del IF: {}", *authority_info.key);
        println!("Segunda parte del IF: {}", Self::authority_id(program_id, farm_id_info.key, farm_data.nonce)?);


        if *authority_info.key != Self::authority_id(program_id, farm_id_info.key, farm_data.nonce)? {
            return Err(FarmError::InvalidProgramAddress.into());
        }

        if amount != FARM_FEE {
            return Err(FarmError::InvalidFarmFee.into());
        }
        msg!("Entra 3");
        Self::token_transfer(
            farm_id_info.key,
            token_program_info.clone(), 
            user_usdc_token_account_info.clone(), 
            fee_owner_info.clone(), 
            user_transfer_authority_info.clone(), 
            farm_data.nonce, 
            amount
        )?;
        msg!("Entra 4");

        farm_data.is_allowed = 1;
        msg!("Entra 5");
        farm_data
            .serialize(&mut *farm_id_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn authority_id(
        program_id: &Pubkey,
        my_info: &Pubkey,
        nonce: u8,
    ) -> Result<Pubkey, FarmError> {
        msg!("Imprime el program id: {}", program_id);
        msg!("Imprime el my_info: {}", my_info);
        msg!("Imprime el nonce: {}", nonce);
        Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[nonce]], program_id)
            .or(Err(FarmError::InvalidProgramAddress))
    }

    pub fn token_transfer<'a>(
        pool: &Pubkey,
        token_program: AccountInfo<'a>,
        source: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        nonce: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        msg!("Token transfer 1");
        let pool_bytes = pool.to_bytes();
        msg!("Token transfer 2");
        let authority_signature_seeds = [&pool_bytes[..32], &[nonce]];
        msg!("Token transfer 3");
        let signers = &[&authority_signature_seeds[..]];
        msg!("Token transfer 4");
        msg!("Token program id: {}", token_program.key);
        msg!("Source pubkey: {}", source.key);
        msg!("Destination pubkey: {}", destination.key);
        msg!("Authority pubkey: {}", authority.key);
        msg!("Amount: {}", amount);
        let ix = spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;
        msg!("Token transfer 5");
        invoke_signed(
            &ix,
            &[source, destination, authority, token_program],
            signers,
        )
    } 
    
}

impl PrintProgramError for FarmError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            FarmError::AlreadyInUse => msg!("Error: The account cannot be initialized because it is already being used"),
            FarmError::InvalidProgramAddress => msg!("Error: The program address provided doesn't match the value generated by the program"),
            FarmError::WrongManager => msg!("Error: Wrong pool manager account"),
            FarmError::SignatureMissing => msg!("Error: Required signature is missing"),
            FarmError::InvalidFeeAccount => msg!("Error: Invalid manager fee account"),
            FarmError::WrongPoolMint => msg!("Error: Specified pool mint account is wrong"),
            FarmError::NotAllowed => msg!("Error: This farm is not allowed yet. The farm creator has to pay additional fee"),
            FarmError::InvalidFarmFee => msg!("Error: Wrong Farm Fee. Farm fee has to be {}",FARM_FEE),
            FarmError::WrongCreator => msg!("Error: Not allowed to create the farm by this creator"),
        }
    }
} 
