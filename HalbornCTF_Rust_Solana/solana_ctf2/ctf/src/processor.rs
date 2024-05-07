use {
    crate::{
        constant::FARM_FEE, error::FarmError, instruction::FarmInstruction, state::Farm
    }, borsh::{BorshDeserialize, BorshSerialize}, num_traits::FromPrimitive, solana_program::{
        account_info::{
            next_account_info,
            AccountInfo,
        }, borsh::try_from_slice_unchecked, decode_error::DecodeError, entrypoint::ProgramResult, instruction::{
            AccountMeta,
            Instruction
        }, msg, program::invoke_signed, program_error::{PrintProgramError, ProgramError}, program_pack::Pack, pubkey::Pubkey
    }, spl_token::{
        instruction::TokenInstruction,
        state::Account as TokenAccount
    }
};

pub struct Processor {}
impl Processor {
    /// this is the instruction data router
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = FarmInstruction::try_from_slice(input)?;
        // here we route the data based on instruction type
        match instruction {
            // pay the farm fee
            FarmInstruction::PayFarmFee(amount) => {
                Self::process_pay_farm_fee(program_id, accounts, amount)
            },

            // otherwise return an error
            _ => Err(FarmError::NotAllowed.into())
        }
    } 

    /// this function handles farm fee payment
    /// by default, farms are not allowed (inactive)
    /// farm creator has to pay 5000 tokens to enable the farm
    pub fn process_pay_farm_fee(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        msg!("Entra");
        let farm_id_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let creator_info = next_account_info(account_info_iter)?;
        let creator_token_account_info = next_account_info(account_info_iter)?;
        let fee_vault_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        msg!("Entra1");
        /* msg!("Farm id info Key {:?}", &farm_id_info.key);
        msg!("Authority info Key {:?}", &authority_info.key);
        msg!("Creator info Key {:?}", &creator_info.key);
        msg!("Creator token info Key {:?}", &creator_token_account_info.key);
        msg!("Fee vault info Key {:?}", &fee_vault_info.key);
        msg!("Token program info Key {:?}", &token_program_info.key); */
        //msg!("FARM ID INFO DATA : {:?}", farm_id_info.data);
        let mut farm_data = try_from_slice_unchecked::<Farm>(&farm_id_info.data.borrow())?;
        msg!("Entra2");
        if farm_data.enabled == 1 {
            return Err(FarmError::AlreadyInUse.into());
        }
        msg!("Entra3");
        if !creator_info.is_signer {
            return Err(FarmError::SignatureMissing.into())
        }
        msg!("Entra4");
        if *creator_info.key != farm_data.creator {
            return Err(FarmError::WrongCreator.into());
        }
        msg!("Entra5");
        if *authority_info.key != Self::authority_id(program_id, farm_id_info.key, farm_data.nonce)? {
            return Err(FarmError::InvalidProgramAddress.into());
        }
        msg!("Entra6");
        if amount != FARM_FEE {
            return Err(FarmError::InvalidFarmFee.into());
        }
        msg!("Entra7");
        //msg!("{:?}", &fee_vault_info.try_borrow_data()?);
        let fee_vault_owner = TokenAccount::unpack_from_slice(&fee_vault_info.try_borrow_data()?)?.owner;

        msg!("Entra8");
        if fee_vault_owner != *authority_info.key {
            return Err(FarmError::InvalidFeeAccount.into())
        }
        msg!("Entra9");
        Self::token_transfer(
            farm_id_info.key,
            token_program_info.clone(), 
            creator_token_account_info.clone(), 
            fee_vault_info.clone(), 
            creator_info.clone(), 
            farm_data.nonce, 
            amount
        )?;
        msg!("Entra10");
        farm_data.enabled = 1;
        msg!("Entra11");
        farm_data
            .serialize(&mut *farm_id_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    /// this function validates the farm authority address
    pub fn authority_id(
        program_id: &Pubkey,
        my_info: &Pubkey,
        nonce: u8,
    ) -> Result<Pubkey, FarmError> {
        Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[nonce]], program_id)
            .or(Err(FarmError::InvalidProgramAddress))
    }

    /// this function facilitates token transfer
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
        let authority_signature_seeds = [&pool_bytes[..32], &[nonce]];
        let signers = &[&authority_signature_seeds[..]];
        msg!("Token transfer 2");
        let data = TokenInstruction::Transfer{amount}.pack();
        msg!("Token transfer 3");
        let mut accounts = Vec::with_capacity(4);
        accounts.push(AccountMeta::new(*source.key, false));
        accounts.push(AccountMeta::new(*destination.key, false));
        accounts.push(AccountMeta::new_readonly(*authority.key, true));
        msg!("Token transfer 4");
        let ix = Instruction {
            program_id: *token_program.key,
            accounts,
            data,
        };
        msg!("Token transfer 5");
        /* msg!("Source: {:?}", source);
        msg!("Destination: {:?}", destination);
        msg!("Authority: {:?}", authority);
        msg!("Token program: {:?}", token_program); */
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
            FarmError::AlreadyInUse => msg!("Error: account already in use"),
            FarmError::InvalidProgramAddress => msg!("Error: the program address provided doesn't match the value generated by the program"),
            FarmError::SignatureMissing => msg!("Error: signature missing"),
            FarmError::InvalidFeeAccount => msg!("Error: fee vault mismatch"),
            FarmError::WrongPoolMint => msg!("Error: pool mint incorrect"),
            FarmError::NotAllowed => msg!("Error: farm not allowed"),
            FarmError::InvalidFarmFee => msg!("Error: farm fee incorrect. should be {}",FARM_FEE),
            FarmError::WrongCreator => msg!("Error: creator mismatch"),
        }
    }
} 
